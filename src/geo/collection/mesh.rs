use super::ds::KDNode;
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
    pub nodes: Vec<KDNode>,
}

impl Mesh {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn new(transform: Transform, path: String, texture: TextureRaw) -> Self {
        macro_rules! n {() => { Vec::new() };}
        let mut ret = Self { transform, path, texture, pos: n!(), norm: n!(), uv: n!(), tri: n!(), pre: n!(), nodes: n!() };
        ret.init();
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
        t_f.iter().for_each(|&(a, b, c)| {
            let g = (gg!(a), gg!(b), gg!(c));
            self.tri.push(g);
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
        KDNode::new_node(self);
    }

    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        KDNode::hit(r, self)
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
