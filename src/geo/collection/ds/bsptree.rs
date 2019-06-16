use super::bbox::BBox;
use crate::{
    geo::{collection::Mesh, HitTemp},
    linalg::{Mat, Ray, Vct},
    Flt, EPS,
};

#[derive(Clone, Debug)]
pub struct Node {
    pub bbox: BBox,
    pub data: Data,
}

#[derive(Clone, Debug)]
pub enum Data {
    A(usize, usize, Vct, Vct), // l, r, pos, norm
    B(Vec<usize>),
}

#[derive(Clone, Debug, Default)]
pub struct BSPTree {
    nodes: Vec<Node>,
}

const K: usize = 8;

impl BSPTree {
    fn _hit(&self, ry: &Ray, mesh: &Mesh) -> Option<HitTemp> {
        let origin = &ry.origin;
        let inv_direct = &Vct::new(1.0 / ry.direct.x, 1.0 / ry.direct.y, 1.0 / ry.direct.z);
        let neg_index = &[inv_direct.x < 0.0, inv_direct.y < 0.0, inv_direct.z < 0.0];
        macro_rules! bbox {
            ($x:expr) => {
                self.nodes[$x].bbox
            };
        }
        if let Some((t_min, t_max)) = bbox!(0).fast_hit(origin, inv_direct, neg_index) {
            let mut ans: Option<HitTemp> = None;
            let mut stk = Vec::new();
            stk.push((0, t_min, t_max));
            while let Some((mut x, mut t_min, mut t_max)) = stk.pop() {
                while let Some((min, max)) = bbox!(x).fast_hit(origin, inv_direct, neg_index) {
                    if t_min < min {
                        t_min = min;
                    }
                    if t_max > max {
                        t_max = max;
                    }
                    if let Some((a, _)) = ans {
                        if a < t_max {
                            t_max = a;
                        }
                    }
                    if t_min > t_max {
                        break;
                    }
                    match &self.nodes[x].data {
                        &Data::A(l, r, p, n) => {
                            let dir = n.dot(ry.direct);
                            if dir.abs() > EPS {
                                let t = n.dot(p - ry.origin) / dir;
                                let (l, r) = if dir >= 0.0 { (l, r) } else { (r, l) };
                                if t <= t_min {
                                    x = r;
                                } else if t >= t_max {
                                    x = l;
                                } else {
                                    stk.push((r, t, t_max));
                                    x = l;
                                    t_max = t;
                                }
                            } else if (ry.origin - p).dot(n) > 0.0 {
                                x = r;
                            } else {
                                stk.push((r, t_min, t_max));
                                x = l;
                            }
                        }
                        &Data::B(ref tri) => {
                            for &i in tri.iter() {
                                mesh.tri_intersect_and_update(i, ry, &mut ans);
                            }
                            break;
                        }
                    }
                }
            }
            return ans;
        }
        None
    }

    fn new_node(&mut self, p: &Vec<Vct>, tri: &mut Vec<(usize, usize, usize, usize)>) -> usize {
        macro_rules! free {
            ($w:expr) => {
                $w.clear();
                $w.shrink_to_fit();
            };
        }
        assert!(tri.len() != 0);
        let bbox = {
            let mut min = Vct::new(1e30, 1e30, 1e30);
            let mut max = Vct::new(-1e30, -1e30, -1e30);
            tri.iter().for_each(|&(a, b, c, _)| {
                min = min.min(p[a]).min(p[b]).min(p[c]);
                max = max.max(p[a]).max(p[b]).max(p[c]);
            });
            BBox { min, max }
        };
        if tri.len() <= K {
            self.nodes.push(Node { bbox, data: Data::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        let avg = |a: usize, b: usize, c: usize, d: usize| (p[a][d] + p[b][d] + p[c][d]) / 3.0;
        let (mut planes, len) = (Vec::new(), tri.len() as Flt);
        let x_mid = tri.iter().fold(0.0, |s, t| s + avg(t.0, t.1, t.2, 0)) / len;
        let y_mid = tri.iter().fold(0.0, |s, t| s + avg(t.0, t.1, t.2, 1)) / len;
        let z_mid = tri.iter().fold(0.0, |s, t| s + avg(t.0, t.1, t.2, 2)) / len;
        let p_mid = Vct::new(x_mid, y_mid, z_mid);
        planes.push((Vct::new(x_mid, 0.0, 0.0), Vct::new(1.0, 0.0, 0.0)));
        planes.push((Vct::new(0.0, y_mid, 0.0), Vct::new(0.0, 1.0, 0.0)));
        planes.push((Vct::new(0.0, 0.0, z_mid), Vct::new(0.0, 0.0, 1.0)));
        let mut dis: Vec<_> = tri
            .iter()
            .map(|&(a, b, c, _)| ((p[a] + p[b] + p[c]) / 3.0 - p_mid).len())
            .enumerate()
            .collect();
        dis.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        macro_rules! rot {
            ($a:expr, $n:expr, $e:expr) => {
                planes.push(($a, $n));
                planes.push(($a, Mat::rot_line_degree($a, $e, 45.0) % $n));
                planes.push(($a, Mat::rot_line_degree($a, $e, -45.0) % $n));
            };
        }
        let mut cal = |a: Vct, b: Vct, c: Vct| {
            let (e1, e2, e3) = (b - a, c - a, c - b);
            let n = e1 % e2;
            planes.push((a, n));
            let (n1, n2, n3) = (n % e1, n % e2, n % e3);
            rot!(a, n1, e1);
            rot!(a, n2, e2);
            rot!(b, n3, e3);
        };
        for i in 0..K {
            let id = dis[i].0;
            let (a, b, c) = (tri[id].0, tri[id].1, tri[id].2);
            cal(p[a], p[b], p[c]);
        }
        for i in 1..=K {
            if tri.len() >= K + i {
                let id = dis[tri.len() - i].0;
                let (a, b, c) = (tri[id].0, tri[id].1, tri[id].2);
                cal(p[a], p[b], p[c]);
            }
        }
        let side = |a: usize, b: usize, c: usize, pos: Vct, norm: Vct| {
            let (a, b, c) = (p[a], p[b], p[c]);
            let x = (a - pos).dot(norm);
            let y = (b - pos).dot(norm);
            let z = (c - pos).dot(norm);
            if x >= 0.0 && y >= 0.0 && z >= 0.0 {
                1
            } else if x <= 0.0 && y <= 0.0 && z <= 0.0 {
                -1
            } else {
                0
            }
        };
        let (mut l, mut r, mut pl) = (Vec::new(), Vec::new(), (Vct::zero(), Vct::zero()));
        let mut init = true;
        planes.iter().for_each(|&(pos, norm)| {
            let (mut tl, mut tr) = (Vec::new(), Vec::new());
            tri.iter().for_each(|&(a, b, c, i)| match side(a, b, c, pos, norm) {
                -1 => tl.push((a, b, c, i)),
                1 => tr.push((a, b, c, i)),
                0 => {
                    tl.push((a, b, c, i));
                    tr.push((a, b, c, i));
                }
                _ => {}
            });
            let eval = (l.len() as isize - r.len() as isize).abs();
            let teval = (tl.len() as isize - tr.len() as isize).abs();
            if init || eval >= teval {
                l = tl;
                r = tr;
                pl = (pos, norm);
                init = false;
            }
        });
        if l.len().max(r.len()) == tri.len() {
            self.nodes.push(Node { bbox, data: Data::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        self.nodes.push(Node { bbox, data: Data::A(0, 0, pl.0, pl.1) });
        let ret = self.nodes.len() - 1;
        free!(tri);
        free!(planes);
        let lc = self.new_node(p, &mut l);
        free!(l);
        let rc = self.new_node(p, &mut r);
        free!(r);
        if let Data::A(ref mut x, ref mut y, _, _) = self.nodes[ret].data {
            *x = lc;
            *y = rc;
        }
        ret
    }

    pub fn build(&mut self, pos: &Vec<Vct>, tri: &Vec<(usize, usize, usize)>) {
        let mut tri: Vec<_> = tri.iter().enumerate().map(|(i, f)| (f.0, f.1, f.2, i)).collect();
        self.new_node(pos, &mut tri);
    }

    pub fn hit(&self, r: &Ray, mesh: &Mesh) -> Option<HitTemp> {
        self._hit(r, mesh)
    }
}
