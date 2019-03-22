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
    X(usize, usize, Vct, Vct, Vec<usize>), // l, r, pos, cross triangle
    A(usize, usize, Vct, Vct),             // l, r, pos, norm
    B(Vec<usize>),
}

#[derive(Clone, Debug, Default)]
pub struct BSPTree {
    nodes: Vec<Node>,
}

const K: usize = 16;
const EP: Flt = 1e-2;

impl BSPTree {
    fn _hit(
        &self,
        x: usize,
        t_min: Flt,
        mut t_max: Flt,
        ry: &Ray,
        inv_direct: &Vct,
        neg_index: &[bool; 3],
        ans: &mut Option<HitTemp>,
        mesh: &Mesh,
    ) {
        if let Some((min, _)) = self.nodes[x].bbox.fast_hit(&ry.origin, inv_direct, neg_index) {
            if let Some((a, _)) = ans {
                if *a + EP < t_max {
                    t_max = *a + EP;
                }
            }
            if t_min <= min && min < t_max {
                match &self.nodes[x].data {
                    &Data::X(l, r, p, n, ref tri) => {
                        for &i in tri.iter() {
                            let (o, d) = (mesh.pre[i] * ry.origin, mesh.pre[i] % ry.direct);
                            let t = -o.z / d.z;
                            if t > EPS && (*ans == None || t < ans.unwrap().0) {
                                let (u, v) = (o.x + t * d.x, o.y + t * d.y);
                                if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
                                    *ans = Some((t, Some((i, u, v))));
                                    if t + EP < t_max {
                                        t_max = t + EP;
                                    }
                                }
                            }
                        }
                        let dir = n.dot(ry.direct);
                        if dir.abs() > EP {
                            let t = n.dot(p - ry.origin) / dir;
                            if (t <= t_min && dir >= 0.0) || (t >= t_max && dir <= 0.0) {
                                self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            } else if (t <= t_min && dir <= 0.0) || (t >= t_max && dir >= 0.0) {
                                self._hit(l, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            } else if dir > 0.0 {
                                self._hit(l, t_min, t + EP, ry, inv_direct, neg_index, ans, mesh);
                                self._hit(r, t - EP, t_max, ry, inv_direct, neg_index, ans, mesh);
                            } else {
                                self._hit(r, t_min, t + EP, ry, inv_direct, neg_index, ans, mesh);
                                self._hit(l, t - EP, t_max, ry, inv_direct, neg_index, ans, mesh);
                            }
                        } else if dir.abs() <= EPS && (ry.origin - p).dot(n) > 0.0 {
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        } else {
                            self._hit(l, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        }
                    }
                    &Data::A(l, r, p, n) => {
                        let dir = n.dot(ry.direct);
                        if dir.abs() > EP {
                            // same as kdtree
                            let t = n.dot(p - ry.origin) / dir;
                            if (t <= t_min && dir >= 0.0) || (t >= t_max && dir <= 0.0) {
                                self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            } else if t <= t_min || t >= t_max {
                                self._hit(l, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                                self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            } else if dir > 0.0 {
                                self._hit(l, t_min, t + EP, ry, inv_direct, neg_index, ans, mesh);
                                self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            } else {
                                self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                                self._hit(l, t - EP, t_max, ry, inv_direct, neg_index, ans, mesh);
                            }
                        } else if dir.abs() <= EPS && (ry.origin - p).dot(n) > 0.0 {
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        } else {
                            self._hit(l, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        }
                    }
                    &Data::B(ref tri) => {
                        for &i in tri.iter() {
                            let (o, d) = (mesh.pre[i] * ry.origin, mesh.pre[i] % ry.direct);
                            let t = -o.z / d.z;
                            if t > EPS && (*ans == None || t < ans.unwrap().0) {
                                let (u, v) = (o.x + t * d.x, o.y + t * d.y);
                                if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
                                    *ans = Some((t, Some((i, u, v))));
                                }
                            }
                        }
                    }
                };
            }
        }
    }

    fn new_node(&mut self, p: &Vec<Vct>, tri: &mut Vec<(usize, usize, usize, usize)>) -> usize {
        assert!(tri.len() != 0);
        macro_rules! free {
            ($w:expr) => {
                $w.clear();
                $w.shrink_to_fit();
            };
        }
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
        let on_left = |a: usize, b: usize, c: usize, pos: Vct, norm: Vct| {
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
        let (mut l, mut r, mut m) = (Vec::new(), Vec::new(), Vec::new());
        let (mut kl, mut kr, mut km) = (Vec::new(), Vec::new(), Vec::new());
        let (mut pl, mut kpl) = ((Vct::zero(), Vct::zero()), (Vct::zero(), Vct::zero()));
        let mut init_k = true;
        let mut init = true;
        planes.iter().for_each(|&(pos, norm)| {
            let (mut tl, mut tr, mut tm) = (Vec::new(), Vec::new(), Vec::new());
            tri.iter().for_each(|&(a, b, c, i)| match on_left(a, b, c, pos, norm) {
                -1 => tl.push((a, b, c, i)),
                1 => tr.push((a, b, c, i)),
                0 => tm.push((a, b, c, i)),
                _ => {}
            });
            let eval = (l.len() as isize - r.len() as isize).abs();
            let teval = (tl.len() as isize - tr.len() as isize).abs();
            if init || eval >= teval {
                l = tl.clone();
                r = tr.clone();
                m = tm.clone();
                pl = (pos, norm);
                init = false;
            }
            if m.len() <= K {
                let keval = (kl.len() as isize - kr.len() as isize).abs();
                if init_k || keval >= teval {
                    kl = tl;
                    kr = tr;
                    km = tm;
                    kpl = (pos, norm);
                    init_k = false;
                }
            }
        });
        if !init_k && (kl.len() as isize - kr.len() as isize).abs() as Flt / len <= 0.1 {
            let km = km.iter().map(|v| v.3).collect();
            self.nodes.push(Node { bbox, data: Data::X(0, 0, kpl.0, kpl.1, km) });
            let ret = self.nodes.len() - 1;
            free!(l);
            free!(r);
            free!(m);
            free!(tri);
            free!(planes);
            let l = self.new_node(p, &mut kl);
            let r = self.new_node(p, &mut kr);
            if let Data::X(ref mut x, ref mut y, _, _, _) = self.nodes[ret].data {
                *x = l;
                *y = r;
            }
            return ret;
        }
        if l.len().max(r.len() + m.len()) == tri.len() {
            self.nodes.push(Node { bbox, data: Data::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        self.nodes.push(Node { bbox, data: Data::A(0, 0, pl.0, pl.1) });
        let ret = self.nodes.len() - 1;
        free!(kl);
        free!(kr);
        free!(km);
        free!(tri);
        free!(planes);
        m.iter().for_each(|v| r.push(*v));
        free!(m);
        let l = self.new_node(p, &mut l);
        let r = self.new_node(p, &mut r);
        if let Data::A(ref mut x, ref mut y, _, _) = self.nodes[ret].data {
            *x = l;
            *y = r;
        }
        ret
    }

    pub fn build(&mut self, pos: &Vec<Vct>, tri: &Vec<(usize, usize, usize)>) {
        let mut tri: Vec<_> = tri.iter().enumerate().map(|(i, f)| (f.0, f.1, f.2, i)).collect();
        self.new_node(pos, &mut tri);
    }

    pub fn hit(&self, r: &Ray, mesh: &Mesh) -> Option<HitTemp> {
        let inv_direct = Vct::new(1.0 / r.direct.x, 1.0 / r.direct.y, 1.0 / r.direct.z);
        let neg_index = [inv_direct.x < 0.0, inv_direct.y < 0.0, inv_direct.z < 0.0];
        let mut ans = None;
        self._hit(0, 0.0, 1e30, r, &inv_direct, &neg_index, &mut ans, mesh);
        ans
    }
}
