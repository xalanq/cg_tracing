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
    fn _hit(&self, x: usize, ry: &Ray, inv_direct: &Vct, ans: &mut Option<HitTemp>, mesh: &Mesh) {
        if let Some((t_min, _)) = self.nodes[x].bbox.hit(&ry.origin, inv_direct) {
            if *ans == None || t_min < ans.unwrap().0 {
                match &self.nodes[x].data {
                    &Data::A(l, r, dim, key) => {
                        let t = (key - ry.origin[dim]) * inv_direct[dim];
                        if t < 0.0 && inv_direct[dim] > 0.0 {
                            self._hit(r, ry, inv_direct, ans, mesh);
                        } else {
                            let (l, r) =
                                if inv_direct[dim] > 0.0 && t > 0.0 { (l, r) } else { (r, l) };
                            self._hit(r, ry, inv_direct, ans, mesh);
                            self._hit(l, ry, inv_direct, ans, mesh);
                        }
                    }
                    &Data::B(ref vi) => {
                        for &i in vi.iter() {
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

    fn new_node(&mut self, p: &Vec<Vct>, tri: &mut [(usize, usize, usize, usize)]) -> usize {
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
        self.nodes.push(Node { bbox, data: Data::A(0, 0, dim, key) });
        let ret = self.nodes.len() - 1;
        let l = self.new_node(p, &mut l);
        let r = self.new_node(p, &mut r);
        if let Data::A(ref mut x, ref mut y, _, _) = self.nodes[ret].data {
            *x = l;
            *y = r;
        };
        ret
    }

    pub fn build(&mut self, pos: &Vec<Vct>, tri: &Vec<(usize, usize, usize)>) {
        let mut tri: Vec<_> = tri.iter().enumerate().map(|(i, f)| (f.0, f.1, f.2, i)).collect();
        self.new_node(pos, &mut tri);
    }

    pub fn hit(&self, r: &Ray, mesh: &Mesh) -> Option<HitTemp> {
        let inv_direct = Vct::new(1.0 / r.direct.x, 1.0 / r.direct.y, 1.0 / r.direct.z);
        let mut ans = None;
        self._hit(0, r, &inv_direct, &mut ans, mesh);
        ans
    }
}
