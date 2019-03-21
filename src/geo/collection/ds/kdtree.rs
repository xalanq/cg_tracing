use super::bbox::BBox;
use crate::{
    geo::{collection::Mesh, HitTemp},
    linalg::{Ray, Vct},
    Flt, EPS,
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

impl KDTree {
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
        if let Some((min, max)) = self.nodes[x].bbox.fast_hit(&ry.origin, inv_direct, neg_index) {
            if let Some((a, _)) = ans {
                if *a < t_max {
                    t_max = *a;
                }
            }
            if t_min <= min && max < t_max {
                match &self.nodes[x].data {
                    &Data::A(l, r, dim, key) => {
                        let dir = inv_direct[dim];
                        let t = (key - ry.origin[dim]) * dir;
                        // 考虑在划分的平面那里剪裁光线线段
                        // 由于划分平面的左边是不会有跨越的三角形的，故可以剪枝
                        if (t <= t_min && dir >= 0.0) || (t >= t_max && dir <= 0.0) {
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        } else if t < t_min || t > t_max {
                            self._hit(l, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        } else if dir >= 0.0 {
                            self._hit(l, t_min, t, ry, inv_direct, neg_index, ans, mesh);
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                        } else {
                            self._hit(r, t_min, t_max, ry, inv_direct, neg_index, ans, mesh);
                            self._hit(l, t, t_max, ry, inv_direct, neg_index, ans, mesh);
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
        if tri.len() <= 16 {
            self.nodes.push(Node { bbox, data: Data::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
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
            if max(a, b, c, dim) < key {
                l.push((a, b, c, i));
            } else {
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
