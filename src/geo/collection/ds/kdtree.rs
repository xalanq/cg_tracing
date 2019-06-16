use super::bbox::BBox;
use crate::{
    geo::{collection::Mesh, HitTemp},
    linalg::{Ray, Vct},
    Flt,
};

#[derive(Clone, Debug)]
pub struct Node {
    pub bbox: BBox,
    pub data: Data,
}

#[derive(Clone, Debug)]
pub enum Data {
    A(usize, usize, usize, Flt), // l, r, dim, key
    B(Vec<usize>),
}

#[derive(Clone, Debug, Default)]
pub struct KDTree {
    nodes: Vec<Node>,
}

const K: usize = 20;

impl KDTree {
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
                        &Data::A(l, r, dim, key) => {
                            let dir = inv_direct[dim];
                            let t = (key - ry.origin[dim]) * dir;
                            let (l, r) = if dir >= 0.0 { (l, r) } else { (r, l) };
                            // 考虑在划分的平面那里剪裁光线线段
                            if t <= t_min {
                                x = r;
                            } else if t >= t_max {
                                x = l;
                            } else {
                                stk.push((r, t, t_max));
                                x = l;
                                t_max = t;
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
        let min = |a: usize, b: usize, c: usize, d: usize| p[a][d].min(p[b][d].min(p[c][d]));
        let max = |a: usize, b: usize, c: usize, d: usize| p[a][d].max(p[b][d].max(p[c][d]));
        let dim = {
            let (mut var, mut avg) = ([0.0; 3], [0.0; 3]);
            tri.iter().for_each(|&(a, b, c, _)| {
                avg.iter_mut().enumerate().for_each(|(d, t)| *t += max(a, b, c, d));
            });
            avg.iter_mut().for_each(|a| *a /= tri.len() as Flt);
            tri.iter().for_each(|&(a, b, c, _)| {
                let f = |(d, t): (usize, &mut Flt)| *t += (max(a, b, c, d) - avg[d]).powi(2);
                var.iter_mut().enumerate().for_each(f);
            });
            var.iter().enumerate().max_by(|x, y| x.1.partial_cmp(y.1).unwrap()).unwrap().0
        };
        tri.sort_by(|&(a, b, c, _), &(x, y, z, _)| {
            max(a, b, c, dim).partial_cmp(&max(x, y, z, dim)).unwrap()
        });
        let mid = tri[tri.len() / 2];
        let key = max(mid.0, mid.1, mid.2, dim);
        let (mut l, mut r) = (Vec::new(), Vec::new());
        tri.iter().for_each(|&(a, b, c, i)| {
            if min(a, b, c, dim) < key {
                l.push((a, b, c, i));
            }
            if max(a, b, c, dim) >= key {
                r.push((a, b, c, i));
            }
        });
        if l.len().max(r.len()) == tri.len() {
            self.nodes.push(Node { bbox, data: Data::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        free!(tri);
        self.nodes.push(Node { bbox, data: Data::A(0, 0, dim, key) });
        let ret = self.nodes.len() - 1;
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
