use super::bbox::BBox;
use crate::{
    geo::{collection::Mesh, HitTemp},
    linalg::{Ray, Vct},
    Flt, EPS,
};

#[derive(Clone, Debug)]
pub struct KDNode {
    pub bbox: BBox,
    pub data: KDNodeType,
}

#[derive(Clone, Debug)]
pub enum KDNodeType {
    A(usize, usize, usize),
    B(Vec<usize>),
}

impl KDNode {
    fn _hit(x: usize, ry: &Ray, inv_direct: &Vct, ans: &mut Option<HitTemp>, mesh: &Mesh) {
        if let Some((t_min, _)) = mesh.nodes[x].bbox.hit(&ry.origin, inv_direct) {
            if *ans == None || t_min < ans.unwrap().0 {
                match &mesh.nodes[x].data {
                    &KDNodeType::A(l, r, dim) => {
                        let (fir, sec) = if inv_direct[dim] > 0.0 { (l, r) } else { (r, l) };
                        Self::_hit(fir, ry, inv_direct, ans, mesh);
                        Self::_hit(sec, ry, inv_direct, ans, mesh);
                    }
                    &KDNodeType::B(ref vi) => {
                        for &i in vi.iter() {
                            let (o, d) = (mesh.pre[i] * ry.origin, mesh.pre[i] % ry.direct);
                            let t = -o.z / d.z;
                            if t > EPS {
                                let (u, v) = (o.x + t * d.x, o.y + t * d.y);
                                if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
                                    if *ans == None || t < ans.unwrap().0 {
                                        *ans = Some((t, Some((i, u, v))));
                                    }
                                }
                            }
                        }
                    }
                };
            }
        }
    }

    pub fn hit(r: &Ray, mesh: &Mesh) -> Option<HitTemp> {
        let inv_direct = Vct::new(1.0 / r.direct.x, 1.0 / r.direct.y, 1.0 / r.direct.z);
        let mut ans = None;
        Self::_hit(0, r, &inv_direct, &mut ans, mesh);
        ans
    }

    pub fn _new_node(
        nodes: &mut Vec<KDNode>,
        p: &Vec<Vct>,
        tri: &mut [(usize, usize, usize, usize)],
    ) -> usize {
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
            nodes.push(KDNode { bbox, data: KDNodeType::B(tri.iter().map(|i| i.3).collect()) });
            return nodes.len() - 1;
        }
        let ctr = |a: usize, b: usize, c: usize, d: usize| (p[a][d] + p[b][d] + p[c][d]) / 3.0;
        let dim = {
            let (mut var, mut avg) = ([0.0; 3], [0.0; 3]);
            tri.iter().for_each(|&(a, b, c, _)| {
                avg.iter_mut().enumerate().for_each(|(d, t)| *t += ctr(a, b, c, d));
            });
            avg.iter_mut().for_each(|a| *a /= tri.len() as Flt);
            tri.iter().for_each(|&(a, b, c, _)| {
                let f = |(d, t): (usize, &mut Flt)| *t += (ctr(a, b, c, d) - avg[d]).powi(2);
                var.iter_mut().enumerate().for_each(f);
            });
            var.iter().enumerate().max_by(|x, y| x.1.partial_cmp(y.1).unwrap()).unwrap().0
        };
        tri.sort_by(|&(a, b, c, _), &(x, y, z, _)| {
            ctr(a, b, c, dim).partial_cmp(&ctr(x, y, z, dim)).unwrap()
        });
        let mid = tri[tri.len() / 2];
        let key = ctr(mid.0, mid.1, mid.2, dim);
        let (mut l, mut r) = (Vec::new(), Vec::new());
        tri.iter().for_each(|&(a, b, c, i)| {
            if p[a][dim].min(p[b][dim]).min(p[c][dim]) < key {
                l.push((a, b, c, i));
            }
            if p[a][dim].max(p[b][dim]).max(p[c][dim]) >= key {
                r.push((a, b, c, i));
            }
        });
        if l.len().max(r.len()) as Flt >= tri.len() as Flt * 0.8 {
            nodes.push(KDNode { bbox, data: KDNodeType::B(tri.iter().map(|i| i.3).collect()) });
            return nodes.len() - 1;
        }
        nodes.push(KDNode { bbox, data: KDNodeType::A(0, 0, dim) });
        let ret = nodes.len() - 1;
        let l = Self::_new_node(nodes, p, &mut l);
        let r = Self::_new_node(nodes, p, &mut r);
        if let KDNodeType::A(ref mut x, ref mut y, _) = nodes[ret].data {
            *x = l;
            *y = r;
        };
        ret
    }

    pub fn new_node(mesh: &mut Mesh) {
        let mut tri: Vec<_> =
            mesh.tri.iter().enumerate().map(|(i, f)| (f.0, f.1, f.2, i)).collect();
        Self::_new_node(&mut mesh.nodes, &mesh.pos, &mut tri);
    }
}
