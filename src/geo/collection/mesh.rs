use crate::{
    geo::{Geo, HitResult, HitTemp, TextureRaw},
    linalg::{Mat, Ray, Transform, Vct},
    Deserialize, Flt, Serialize, EPS,
};
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub transform: Transform,
    pub path: String,
    pub texture: TextureRaw,
    #[serde(skip_serializing, skip_deserializing)]
    pub pos: Vec<Vct>,
    #[serde(skip_serializing, skip_deserializing)]
    pub norm: Vec<Vct>,
    #[serde(skip_serializing, skip_deserializing)]
    pub uv: Vec<(Flt, Flt)>,
    #[serde(skip_serializing, skip_deserializing)]
    pub tri: Vec<(usize, usize, usize)>,
    #[serde(skip_serializing, skip_deserializing)]
    pub pre: Vec<Mat>,
    #[serde(skip_serializing, skip_deserializing)]
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, Default)]
pub struct Bbox {
    pub min: Vct,
    pub max: Vct,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub bbox: Bbox,
    pub data: NodeType,
}

#[derive(Clone, Debug)]
pub enum NodeType {
    A(usize, usize, usize, Flt),
    B(Vec<usize>),
}

impl Bbox {
    fn hit(&self, origin: Vct, inv_direct: Vct) -> Option<(Flt, Flt)> {
        let a = (self.min - origin) * inv_direct;
        let b = (self.max - origin) * inv_direct;
        let min = a.min(b);
        let max = a.max(b);
        let t_min = min.x.max(min.y).max(min.z).max(0.0);
        let t_max = max.x.min(max.y).min(max.z);
        if t_min < t_max {
            Some((t_min, t_max))
        } else {
            None
        }
    }
}

impl Node {
    fn hit(
        x: usize,
        ry: &Ray,
        inv_direct: Vct,
        t_min: Flt,
        t_max: Flt,
        mesh: &Mesh,
    ) -> Option<HitTemp> {
        if let Some((_t_min, _t_max)) = mesh.nodes[x].bbox.hit(ry.origin, inv_direct) {
            let (t_min, t_max) = (t_min.min(_t_min), t_max.min(_t_max));
            match &mesh.nodes[x].data {
                &NodeType::A(l, r, dim, key) => {
                    let t = (key - ry.origin[dim]) * inv_direct[dim];
                    let (fir, sec) = if inv_direct[dim] > 0.0 { (l, r) } else { (r, l) };
                    if t > t_max {
                        return Self::hit(fir, ry, inv_direct, t_min, t_max, mesh);
                    }
                    if t < t_min {
                        return Self::hit(sec, ry, inv_direct, t_min, t_max, mesh);
                    }
                    return match Self::hit(fir, ry, inv_direct, t_min, t, mesh) {
                        Some(o) => Some(o),
                        None => Self::hit(sec, ry, inv_direct, t, t_max, mesh),
                    };
                }
                &NodeType::B(ref vi) => {
                    let (mut ans, mut best) = (None, 1e30);
                    for &i in vi.iter() {
                        let (o, d) = (mesh.pre[i] * ry.origin, mesh.pre[i] % ry.direct);
                        let t = -o.z / d.z;
                        if t > EPS {
                            let (u, v) = (o.x + t * d.x, o.y + t * d.y);
                            if u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t < best {
                                best = t;
                                ans = Some((t, Some((i, u, v))));
                            }
                        }
                    }
                    return ans;
                }
            };
        }
        None
    }
}

impl Mesh {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn new(transform: Transform, path: String, texture: TextureRaw) -> Self {
        macro_rules! n {() => { Vec::new() };}
        let mut ret = Self { transform, path, texture, pos: n!(), norm: n!(), uv: n!(), tri: n!(), pre: n!(), nodes: n!() };
        ret.init();
        ret
    }

    fn new_node(&mut self, tri: &mut [(usize, usize, usize, usize)]) -> usize {
        assert!(tri.len() != 0);
        let pos = &self.pos;
        let bbox = {
            let mut min = Vct::new(1e30, 1e30, 1e30);
            let mut max = Vct::new(-1e30, -1e30, -1e30);
            tri.iter().for_each(|&(a, b, c, _)| {
                min = min.min(pos[a]).min(pos[b]).min(pos[c]);
                max = max.max(pos[a]).max(pos[b]).max(pos[c]);
            });
            Bbox { min, max }
        };
        if tri.len() <= 16 {
            self.nodes.push(Node { bbox, data: NodeType::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        let mind = |a: usize, b: usize, c: usize, d: usize| pos[a][d].min(pos[b][d]).min(pos[c][d]);
        let dim = {
            let (mut var, mut avg) = ([0.0; 3], [0.0; 3]);
            tri.iter().for_each(|&(a, b, c, _)| {
                avg.iter_mut().enumerate().for_each(|(d, t)| *t += mind(a, b, c, d));
            });
            avg.iter_mut().for_each(|a| *a /= tri.len() as Flt);
            tri.iter().for_each(|&(a, b, c, _)| {
                let f = |(d, t): (usize, &mut Flt)| *t += (mind(a, b, c, d) - avg[d]).powi(2);
                var.iter_mut().enumerate().for_each(f);
            });
            var.iter().enumerate().max_by(|x, y| x.1.partial_cmp(y.1).unwrap()).unwrap().0
        };
        tri.sort_by(|&(a, b, c, _), &(x, y, z, _)| {
            mind(a, b, c, dim).partial_cmp(&mind(x, y, z, dim)).unwrap()
        });
        let mid = tri[tri.len() / 2];
        let key = mind(mid.0, mid.1, mid.2, dim);
        let (mut l, mut r) = (Vec::new(), Vec::new());
        tri.iter().for_each(|&(a, b, c, i)| {
            if mind(a, b, c, dim) < key {
                l.push((a, b, c, i));
            }
            if pos[a][dim].max(pos[b][dim]).max(pos[c][dim]) >= key {
                r.push((a, b, c, i));
            }
        });
        if l.len().max(r.len()) == tri.len() {
            self.nodes.push(Node { bbox, data: NodeType::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        self.nodes.push(Node { bbox, data: NodeType::A(0, 0, dim, key) });
        let ret = self.nodes.len() - 1;
        let l = self.new_node(&mut l);
        let r = self.new_node(&mut r);
        if let NodeType::A(ref mut x, ref mut y, _, _) = self.nodes[ret].data {
            *x = l;
            *y = r;
        };
        ret
    }
}

impl Geo for Mesh {
    fn init(&mut self) {
        let file = File::open(&self.path).expect(&format!("Cannot open {}", self.path));
        let (mut t_v, mut t_vt, mut t_vn, mut t_f) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for line in BufReader::new(file).lines() {
            let line = line.expect("Failed to load mesh object");
            let mut w = line.split_whitespace();
            macro_rules! nx {
                ($w:expr) => {
                    $w.next().unwrap().parse().unwrap()
                };
            }
            macro_rules! nxt {
                ($w:expr, $t:ty) => {
                    $w.next().unwrap().parse::<$t>().unwrap()
                };
            }
            macro_rules! nxtf {
                ($w:expr) => {{
                    let mut a = Vec::new();
                    $w.next().unwrap().split('/').for_each(|x| {
                        if let Ok(i) = x.parse::<usize>() {
                            a.push(i);
                        }
                    });
                    match a.len() {
                        2 => (a[0], 0, a[1]),
                        3 => (a[0], a[1], a[2]),
                        _ => panic!("The mesh object has non-triangle"),
                    }
                }};
            }
            match w.next() {
                Some("v") => t_v.push(self.transform.value * Vct::new(nx!(w), nx!(w), nx!(w))),
                Some("vt") => t_vt.push((nxt!(w, Flt), nxt!(w, Flt))),
                Some("vn") => t_vn.push(self.transform.value % Vct::new(nx!(w), nx!(w), nx!(w))),
                Some("f") => t_f.push((nxtf!(w), nxtf!(w), nxtf!(w))),
                _ => (),
            }
        }
        let mut vis = HashMap::new();
        macro_rules! gg {
            ($a:expr) => {{
                *vis.entry($a).or_insert_with(|| {
                    self.pos.push(t_v[$a.0 - 1]);
                    self.uv.push(if $a.1 != 0 { t_vt[$a.1 - 1] } else { (-1.0, -1.0) });
                    self.norm.push(t_vn[$a.2 - 1]);
                    self.pos.len() - 1
                })
            }};
        }
        let mut tri: Vec<(usize, usize, usize, usize)> = Vec::new();
        t_f.iter().for_each(|&(a, b, c)| {
            let g = (gg!(a), gg!(b), gg!(c));
            self.tri.push(g);
            tri.push((g.0, g.1, g.2, self.tri.len() - 1));
            let (v1, v2, v3) = (self.pos[g.0], self.pos[g.1], self.pos[g.2]);
            let (e1, e2) = (v2 - v1, v3 - v1);
            let n = e1 % e2;
            let ni = Vct::new(1.0 / n.x, 1.0 / n.y, 1.0 / n.z);
            let nv = v1.dot(n);
            let (x2, x3) = (v2 % v1, v3 % v1);
            #[cfg_attr(rustfmt, rustfmt_skip)]
            self.pre.push({
                if n.x.abs() > n.y.abs().max(n.z.abs()) {
                    Mat {
                        m00: 0.0, m01: e2.z * ni.x,  m02: -e2.y * ni.x, m03: x3.x * ni.x,
                        m10: 0.0, m11: -e1.z * ni.x, m12: e1.y * ni.x,  m13: -x2.x * ni.x,
                        m20: 1.0, m21: n.y * ni.x,   m22: n.z * ni.x,   m23: -nv * ni.x,
                        m33: 1.0, ..Default::default()
                    }
                } else if n.y.abs() > n.z.abs() {
                    Mat {
                        m00: -e2.z * ni.y, m01: 0.0, m02: e2.x * ni.y,  m03: x3.y * ni.y,
                        m10: e1.z * ni.y,  m11: 0.0, m12: -e1.x * ni.y, m13: -x2.y * ni.y,
                        m20: n.x * ni.y,   m21: 1.0, m22: n.z * ni.y,   m23: -nv * ni.y,
                        m33: 1.0, ..Default::default()
                    }
                } else if n.z.abs() > EPS {
                    Mat {
                        m00: e2.y * ni.z,  m01: -e2.x * ni.z, m02: 0.0, m03: x3.z * ni.z,
                        m10: -e1.y * ni.z, m11: e1.x * ni.z,  m12: 0.0, m13: -x2.z * ni.z,
                        m20: n.x * ni.z,   m21: n.y * ni.z,   m22: 1.0, m23: -nv * ni.z,
                        m33: 1.0, ..Default::default()
                    }
                } else {
                    panic!("Invalid triangle");
                }
            });
        });
        self.new_node(&mut tri);
    }

    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        let inv_direct = Vct::new(1.0 / r.direct.x, 1.0 / r.direct.y, 1.0 / r.direct.z);
        Node::hit(0, r, inv_direct, 0.0, 1e30, self)
    }

    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        let (i, u, v) = tmp.1.unwrap();
        let (a, b, c) = self.tri[i];
        HitResult {
            pos: r.origin + r.direct * tmp.0,
            norm: self.norm[a] * (1.0 - u - v) + self.norm[b] * u + self.norm[c] * v,
            texture: self.texture,
        }
    }
}
