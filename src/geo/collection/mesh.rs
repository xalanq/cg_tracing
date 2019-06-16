use super::ds::{BSPTree, KDTree, MyTree};
use crate::{
    geo::{Geo, HitResult, HitTemp, TextureRaw},
    linalg::{Mat, Ray, Transform, Vct},
    Deserialize, Flt, Serialize, EPS,
};
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Serialize, Deserialize)]
pub enum TreeType {
    KDTree,
    BSPTree,
    MyTree,
}

#[derive(Clone, Debug)]
pub enum Tree {
    KDTree(KDTree),
    BSPTree(BSPTree),
    MyTree(MyTree),
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub path: String,
    pub texture: TextureRaw,
    pub transform: Transform,
    pub pos: Vec<Vct>,
    pub norm: Vec<Vct>,
    pub uv: Vec<(Flt, Flt)>,
    pub tri: Vec<(usize, usize, usize)>,
    pub pre: Vec<Mat>,
    pub tree: Tree,
}

impl Mesh {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn new(path: String, texture: TextureRaw, transform: Transform, tree_type: TreeType) -> Self {
        let (pos, norm, uv, tri, pre, tree) = Self::load(&path, &transform, tree_type);
        Self { path, texture, transform, pos, norm, uv, tri, pre, tree }
    }

    fn load(
        path: &str,
        transform: &Transform,
        tree_type: TreeType,
    ) -> (Vec<Vct>, Vec<Vct>, Vec<(Flt, Flt)>, Vec<(usize, usize, usize)>, Vec<Mat>, Tree) {
        let file = File::open(path).expect(&format!("Cannot open {}", path));
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
                Some("v") => wp!(t_v.push(transform.value * Vct::new(nx!(), nx!(), nx!()))),
                Some("vt") => wp!(t_vt.push((nxt!(Flt), nxt!(Flt)))),
                Some("vn") => {
                    wp!(t_vn.push((transform.value % Vct::new(nx!(), nx!(), nx!())).norm()))
                }
                Some("f") => wp!(t_f.push((nxtf!(), nxtf!(), nxtf!()))),
                _ => (),
            }
        }
        let mut vis = HashMap::new();
        let (mut pos, mut uv, mut norm, mut tri, mut pre) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        macro_rules! gg {
            ($a:expr) => {{
                *vis.entry($a).or_insert_with(|| {
                    pos.push(t_v[$a.0 - 1]);
                    uv.push(if $a.1 != 0 { t_vt[$a.1 - 1] } else { (-1.0, -1.0) });
                    norm.push(t_vn[$a.2 - 1]);
                    pos.len() - 1
                })
            }};
        }
        t_f.iter().for_each(|&(a, b, c)| {
            let g = (gg!(a), gg!(b), gg!(c));
            tri.push(g);
            let (v1, v2, v3) = (pos[g.0], pos[g.1], pos[g.2]);
            let (e1, e2) = (v2 - v1, v3 - v1);
            let n = e1 % e2;
            let ni = Vct::new(1.0 / n.x, 1.0 / n.y, 1.0 / n.z);
            let nv = v1.dot(n);
            let (x2, x3) = (v2 % v1, v3 % v1);
            #[cfg_attr(rustfmt, rustfmt_skip)]
            pre.push({
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
        let tree = match tree_type {
            TreeType::KDTree => {
                let mut ret = KDTree::default();
                ret.build(&pos, &tri);
                Tree::KDTree(ret)
            }
            TreeType::BSPTree => {
                let mut ret = BSPTree::default();
                ret.build(&pos, &tri);
                Tree::BSPTree(ret)
            }
            TreeType::MyTree => {
                let mut ret = MyTree::default();
                ret.build(&pos, &tri);
                Tree::MyTree(ret)
            }
        };
        (pos, norm, uv, tri, pre, tree)
    }

    pub fn tri_intersect_and_update(&self, i: usize, r: &Ray, ans: &mut Option<HitTemp>) {
        let (m, o, d) = (&self.pre[i], r.origin, r.direct);
        let dz = m.m20 * d.x + m.m21 * d.y + m.m22 * d.z;
        if dz.abs() <= EPS {
            return;
        }
        let oz = m.m20 * o.x + m.m21 * o.y + m.m22 * o.z + m.m23;
        let t = -oz / dz;
        if t > EPS && (ans.is_none() || t < ans.unwrap().0) {
            let hit = o + d * t;
            let u = m.m00 * hit.x + m.m01 * hit.y + m.m02 * hit.z + m.m03;
            if u < 0.0 || u > 1.0 {
                return;
            }
            let v = m.m10 * hit.x + m.m11 * hit.y + m.m12 * hit.z + m.m13;
            if v < 0.0 || u + v > 1.0 {
                return;
            }
            *ans = Some((t, Some((i, u, v))));
        }
    }
}

impl Geo for Mesh {
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        match &self.tree {
            &Tree::KDTree(ref t) => t.hit(r, self),
            &Tree::BSPTree(ref t) => t.hit(r, self),
            &Tree::MyTree(ref t) => t.hit(r, self),
        }
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

impl Serialize for Mesh {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("mesh", 3)?;
        s.serialize_field("path", &self.path)?;
        s.serialize_field("texture", &self.texture)?;
        s.serialize_field("transform", &self.transform)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MeshVisitor;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Path,
            Texture,
            Transform,
            TreeType,
            Type,
        }

        impl<'de> Visitor<'de> for MeshVisitor {
            type Value = Mesh;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Mesh")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Mesh, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut path = None;
                let mut texture = None;
                let mut transform = None;
                let mut tree_type = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::Texture => {
                            if texture.is_some() {
                                return Err(de::Error::duplicate_field("texture"));
                            }
                            texture = Some(map.next_value()?);
                        }
                        Field::Transform => {
                            if transform.is_some() {
                                return Err(de::Error::duplicate_field("transform"));
                            }
                            transform = Some(map.next_value()?);
                        }
                        Field::TreeType => {
                            if tree_type.is_some() {
                                return Err(de::Error::duplicate_field("tree_type"));
                            }
                            tree_type = Some(map.next_value()?);
                        }
                        Field::Type => {}
                    }
                }
                let path = path.ok_or_else(|| de::Error::missing_field("path"))?;
                let texture = texture.ok_or_else(|| de::Error::missing_field("texture"))?;
                let transform = transform.ok_or_else(|| de::Error::missing_field("transform"))?;
                let tree_type = tree_type.ok_or_else(|| de::Error::missing_field("tree_type"))?;
                Ok(Mesh::new(path, texture, transform, tree_type))
            }
        }

        deserializer.deserialize_map(MeshVisitor {})
    }
}
