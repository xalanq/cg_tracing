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
    A(usize, usize, usize),
    B(Vec<usize>),
}

impl Bbox {
    fn hit(&self, origin: &Vct, inv_direct: &Vct) -> Option<(Flt, Flt)> {
        let a = (self.min - *origin) * *inv_direct;
        let b = (self.max - *origin) * *inv_direct;
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
    fn hit(x: usize, ry: &Ray, inv_direct: &Vct, ans: &mut Option<HitTemp>, mesh: &Mesh) {
        if let Some((t_min, _)) = mesh.nodes[x].bbox.hit(&ry.origin, inv_direct) {
            if *ans == None || t_min < ans.unwrap().0 {
                match &mesh.nodes[x].data {
                    &NodeType::A(l, r, dim) => {
                        let (fir, sec) = if inv_direct[dim] > 0.0 { (l, r) } else { (r, l) };
                        Self::hit(fir, ry, inv_direct, ans, mesh);
                        Self::hit(sec, ry, inv_direct, ans, mesh);
                    }
                    &NodeType::B(ref vi) => {
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
        let p = &self.pos;
        let bbox = {
            let mut min = Vct::new(1e30, 1e30, 1e30);
            let mut max = Vct::new(-1e30, -1e30, -1e30);
            tri.iter().for_each(|&(a, b, c, _)| {
                min = min.min(p[a]).min(p[b]).min(p[c]);
                max = max.max(p[a]).max(p[b]).max(p[c]);
            });
            Bbox { min, max }
        };
        if tri.len() <= 16 {
            self.nodes.push(Node { bbox, data: NodeType::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
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
        let mut same = 0;
        tri.iter().for_each(|&(a, b, c, i)| {
            let mut cnt = 0;
            if p[a][dim].min(p[b][dim]).min(p[c][dim]) < key {
                l.push((a, b, c, i));
                cnt += 1;
            }
            if p[a][dim].max(p[b][dim]).max(p[c][dim]) >= key {
                r.push((a, b, c, i));
                cnt += 1;
            }
            if cnt == 2 {
                same += 1;
            }
        });
        if same as Flt / tri.len() as Flt >= 0.5 {
            self.nodes.push(Node { bbox, data: NodeType::B(tri.iter().map(|i| i.3).collect()) });
            return self.nodes.len() - 1;
        }
        self.nodes.push(Node { bbox, data: NodeType::A(0, 0, dim) });
        let ret = self.nodes.len() - 1;
        let l = self.new_node(&mut l);
        let r = self.new_node(&mut r);
        if let NodeType::A(ref mut x, ref mut y, _) = self.nodes[ret].data {
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
                () => {
                    w.next().unwrap().parse().unwrap()
                };
            }
            macro_rules! nxt {
                ($t:ty) => {
                    w.next().unwrap().parse::<$t>().unwrap()
                };
            }
            macro_rules! nxtf {
                () => {{
                    let mut a = Vec::new();
                    w.next().unwrap().split('/').for_each(|x| {
                        if let Ok(i) = x.parse::<usize>() {
                            a.push(i);
                        }
                    });
                    match a.len() {
                        2 => (a[0], 0, a[1]),
                        3 => (a[0], a[1], a[2]),
                        _ => panic!("invalid vertex of a face"),
                    }
                }};
            }
            macro_rules! wp {
                ($e:expr) => {{
                    $e;
                    w.next().map(|_| panic!("The mesh object has a non-triangle"));
                }};
            }
            match w.next() {
                Some("v") => wp!(t_v.push(self.transform.value * Vct::new(nx!(), nx!(), nx!()))),
                Some("vt") => wp!(t_vt.push((nxt!(Flt), nxt!(Flt)))),
                Some("vn") => wp!(t_vn.push(self.transform.value % Vct::new(nx!(), nx!(), nx!()))),
                Some("f") => wp!(t_f.push((nxtf!(), nxtf!(), nxtf!()))),
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
        let mut ans = None;
        Node::hit(0, r, &inv_direct, &mut ans, self);
        ans
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
