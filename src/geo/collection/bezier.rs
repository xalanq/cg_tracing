use super::ds::BBox;
use crate::{
    geo::{Geo, HitResult, HitTemp, Material, Texture, TextureRaw},
    linalg::{Ray, Transform, Vct},
    Deserialize, Flt, Serialize, EPS, PI,
};
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Bezier2D {
    pub n: usize,
    pub a: Vec<(Flt, Flt)>,
}

impl Bezier2D {
    pub fn new(point: &Vec<(Flt, Flt)>) -> Self {
        let n = point.len() - 1;
        let mut a = vec![];
        let mut x = point.iter().map(|p| p.0).collect::<Vec<_>>();
        let mut y = point.iter().map(|p| p.1).collect::<Vec<_>>();
        let mut t = 1.0;
        for i in 0..=n {
            a.push((x[0] * t, y[0] * t));
            t = t * (n - i) as Flt / (i + 1) as Flt;
            for j in 0..n - i {
                x[j] = x[j + 1] - x[j];
                y[j] = y[j + 1] - y[j];
            }
        }
        Bezier2D { n, a }
    }

    pub fn p(&self, t: Flt) -> (Flt, Flt) {
        let (mut x, mut y) = (0.0, 0.0);
        for i in (0..=self.n).rev() {
            x = self.a[i].0 + x * t;
            y = self.a[i].1 + y * t;
        }
        (x, y)
    }

    pub fn dp(&self, t: Flt) -> (Flt, Flt) {
        let (mut x, mut y) = (0.0, 0.0);
        for i in (1..=self.n).rev() {
            x = self.a[i].0 * i as Flt + x * t;
            y = self.a[i].1 * i as Flt + y * t;
        }
        (x, y)
    }
}

#[derive(Clone, Debug)]
pub struct BezierRotate {
    pub point: Vec<(Flt, Flt)>, // 绕 y 轴（x = 0）旋转，给的点尽量从下往上给
    pub texture: Texture,
    pub transform: Transform,
    b2d: Bezier2D,
    bbox: BBox,
}

impl BezierRotate {
    pub fn new(point: Vec<(Flt, Flt)>, texture: Texture, transform: Transform) -> Self {
        let b2d = Bezier2D::new(&point);
        let (mut max_x, mut max_y, mut min_y) = (point[0].0.abs(), point[0].1, point[0].1);
        for i in 1..point.len() {
            max_x = max_x.max(point[i].0.abs());
            max_y = max_y.max(point[i].1);
            min_y = min_y.min(point[i].1);
        }
        let bbox =
            BBox { max: Vct::new(max_x, max_y, max_x), min: Vct::new(-max_x, min_y, -max_x) };
        Self { point, texture, transform, b2d, bbox }
    }
}

impl Geo for BezierRotate {
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        let (o, d) = (self.transform.inv * r.origin, (self.transform.inv % r.direct).norm());
        if self.bbox.hit(&o, &d).is_some() {
            let t1 = o.x * d.y - d.x * o.y;
            let t2 = o.z * d.y - d.z * o.y;
            let a = d.x * d.x + d.z * d.z;
            let a2 = 2.0 * a;
            let b = 2.0 * (t1 * d.x + t2 * d.z);
            let c = t1 * t1 + t2 * t2;
            let w = -d.y * d.y;
            let w2 = 2.0 * w;
            let (mut a2i, mut a4, mut bb, mut bb2, mut cc) = (0.0, 0.0, 0.0, 0.0, 0.0);
            if d.y.abs() < EPS {
                bb = 2.0 * (o.x * d.x + o.z * d.z);
                bb2 = bb * bb;
                a2i = 1.0 / a2;
                a4 = 4.0 * a;
                cc = o.x * o.x + o.z * o.z;
            }
            let mut ans: Option<HitTemp> = None;
            let sample = self.b2d.n * 2;
            let step = 1.0 / sample as Flt;
            for i in 0..=sample {
                let mut t = i as Flt * step;
                let (mut x, mut y) = self.b2d.p(t);
                let mut xx = x * x;
                let mut f = (a * y + b) * y + c + w * xx;
                for _ in 0..15 {
                    // 一定要足够
                    if f.abs() < 1e-12 {
                        // 一定要足够小
                        let k = if d.y.abs() < EPS {
                            // 圆交点，半径为 x
                            let ccc = cc - xx;
                            let delta = (bb2 - a4 * ccc).sqrt();
                            let t1 = (-bb - delta) * a2i;
                            let t2 = (-bb + delta) * a2i;
                            if t1 < t2 && t1 > EPS {
                                t1
                            } else {
                                t2
                            }
                        } else {
                            (y - o.y) / d.y
                        };
                        if k > EPS && (ans.is_none() || ans.unwrap().0 > k) {
                            let px = o.x + k * d.x;
                            let pz = o.z + k * d.z;
                            if (px * px + pz * pz - xx).abs() < EPS {
                                ans = Some((k, Some((0, t, x))));
                            }
                        }
                        break;
                    }
                    let (dx, dy) = self.b2d.dp(t);
                    let df = (a2 * y + b) * dy + w2 * x * dx;
                    let g = -f / df;
                    let mut lambda = 1.0;
                    let weight = if t < 0.1 || t > 0.9 { 0.9 } else { 0.5 };
                    let (mut t1, mut f1) = (0.0, 0.0);
                    // 下山牛顿迭代
                    while lambda > 1e-5 {
                        t1 = t + lambda * g;
                        if t1 < 0.0 || t1 > 1.0 {
                            lambda *= weight;
                            continue;
                        }
                        let xy = self.b2d.p(t1);
                        x = xy.0;
                        y = xy.1;
                        xx = x * x;
                        f1 = (a * y + b) * y + c + w * xx;
                        if f1.abs() < f.abs() {
                            break;
                        }
                        lambda *= weight;
                    }
                    if t1 < 0.0 || t1 > 1.0 || (f1.abs() >= 1e-10 && (f1 - f).abs() < 1e-12) {
                        break;
                    }
                    t = t1;
                    f = f1;
                }
            }
            return ans;
        }
        None
    }

    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        let (o, d) = (self.transform.inv * r.origin, (self.transform.inv % r.direct).norm());
        let k = tmp.0;
        let (_, t, x) = tmp.1.unwrap();
        let (mut cos, mut sin) = (1.0, 0.0);
        let norm = (self.transform.value
            % if x.abs() < EPS {
                Vct::new(0.0, -d.y, 0.0)
            } else {
                cos = (o.x + k * d.x) / x;
                sin = (o.z + k * d.z) / x;
                let (dx, dy) = self.b2d.dp(t);
                let dt = Vct::new(cos * dx, dy, sin * dx);
                let dd = Vct::new(-sin * x, 0.0, cos * x);
                dt % dd
            })
        .norm();
        HitResult {
            pos: r.origin + r.direct * k,
            norm,
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    if cos < -1.0 {
                        cos = -1.0;
                    } else if cos > 1.0 {
                        cos = 1.0;
                    }
                    let mut px = cos.acos();
                    if sin < 0.0 {
                        px += PI;
                    }
                    px = (px / PI / 2.0) * img.image.w as Flt;
                    let py = t * img.image.h as Flt;
                    let col = img.image.get_repeat(px as isize, py as isize);
                    TextureRaw {
                        emission: Vct::zero(),
                        color: Vct::new(col.0, col.1, col.2),
                        material: if col.3 > 0.0 { Material::Diffuse } else { img.material },
                    }
                }
            },
        }
    }
}

impl Serialize for BezierRotate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("bezier_rotate", 3)?;
        s.serialize_field("point", &self.point)?;
        s.serialize_field("texture", &self.texture)?;
        s.serialize_field("transform", &self.transform)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for BezierRotate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BezierRotateVisitor;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Point,
            Texture,
            Transform,
            Type,
        }

        impl<'de> Visitor<'de> for BezierRotateVisitor {
            type Value = BezierRotate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("BezierRotate")
            }

            fn visit_map<V>(self, mut map: V) -> Result<BezierRotate, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut point = None;
                let mut texture = None;
                let mut transform = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Point => {
                            if point.is_some() {
                                return Err(de::Error::duplicate_field("point"));
                            }
                            point = Some(map.next_value()?);
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
                        Field::Type => {}
                    }
                }
                let point = point.ok_or_else(|| de::Error::missing_field("point"))?;
                let texture = texture.ok_or_else(|| de::Error::missing_field("texture"))?;
                let transform = transform.ok_or_else(|| de::Error::missing_field("transform"))?;
                Ok(BezierRotate::new(point, texture, transform))
            }
        }

        deserializer.deserialize_map(BezierRotateVisitor {})
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier2d() {
        let testit = |point: Vec<(Flt, Flt)>| {
            let n = point.len() - 1;
            let mut fac = vec![1];
            for i in 1..=n {
                fac.push(fac[i - 1] * i);
            }
            let p = |t: Flt| -> (Flt, Flt) {
                let (mut x, mut y) = (0.0, 0.0);
                for i in 0..=n {
                    let a = (fac[n] / fac[i] / fac[n - i]) as Flt
                        * t.powi(i as i32)
                        * (1.0 - t).powi((n - i) as i32);
                    x += point[i].0 as Flt * a;
                    y += point[i].1 as Flt * a;
                }
                (x, y)
            };
            let b = Bezier2D::new(&point);
            for i in 0..=10 {
                let t = i as Flt / 10.0;
                let p1 = p(t);
                let p2 = b.p(t);
                assert!((p1.0 - p2.0).abs() < 1e-5);
                assert!((p1.1 - p2.1).abs() < 1e-5);
            }
        };
        testit(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)]);
        testit(vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0), (10.0, 20.0), (0.0, 20.0)]);
    }

    use std::fs::File;
    use std::io::Write;

    #[test]
    fn draw() {
        let drawit = |point: Vec<(Flt, Flt)>, r: &Ray| {
            let (o, d) = (r.origin, r.direct);
            let b2d = Bezier2D::new(&point);
            /*
            let t1 = o.x * d.y - d.x * o.y;
            let t2 = o.z * d.y - d.z * o.y;
            let a = d.x * d.x + d.z * d.z;
            let b = 2.0 * (t1 * d.x + t2 * d.z);
            let c = t1 * t1 + t2 * t2;
            let w = -d.y * d.y;
            let ans: Option<HitTemp> = None;
            */
            let sample = 1000;
            let mut xy_file = File::create("result/xy.txt").unwrap();
            let mut f_file = File::create("result/f.txt").unwrap();
            let mut df_file = File::create("result/df.txt").unwrap();
            // let mut ab_file = File::create("result/ab.txt").unwrap();
            for i in 0..=sample {
                /*
                let t = i as Flt / sample as Flt;
                let (x, y) = b2d.p(t);
                let f = (a * y + b) * y + c + w * x * x;
                let (dx, dy) = b2d.dp(t);
                let df = (2.0 * a * y + b) * dy + 2.0 * w * x * dx;
                */
                let t = i as Flt / sample as Flt;
                let (x, y) = b2d.p(t);
                let a = o.x * d.y + (y - o.y) * d.x;
                let b = o.z * d.y + (y - o.y) * d.z;
                let g = (a * a + b * b).sqrt();
                let f = g - d.y * x;
                let (dx, dy) = b2d.dp(t);
                let df = if a.abs() <= EPS && b.abs() <= EPS {
                    -d.y * dx
                } else {
                    (a * d.x + b * d.z) * dy / g - d.y * dx
                };
                writeln!(&mut xy_file, "{} {}", x, y).unwrap();
                writeln!(&mut f_file, "{} {}", t, f).unwrap();
                writeln!(&mut df_file, "{} {}", t, df).unwrap();
                // writeln!(&mut ab_file, "{} {} {}", t, (a * y + b) * y + c, -w * x * x).unwrap();
            }
        };
        drawit(
            vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0), (10.0, 20.0), (0.0, 20.0)],
            // &Ray::new(Vct::new(15.0, 20.0, 0.0), Vct::new(-1.0, -1.0, 0.0).norm()),
            &Ray::new(Vct::new(5.0, 5.0, 50.0), Vct::new(0.0, 0.0, -1.0).norm()),
        );
    }

    use crate::geo::Material;
    #[test]
    fn check_norm() {
        let drawit = |point: Vec<(Flt, Flt)>, r: &Ray| {
            let texture = Texture::Raw(TextureRaw::new(
                Vct::zero(),
                Vct::new(1.0, 1.0, 1.0),
                Material::Specular,
            ));
            let transform = Transform::new(Vec::new());
            let b2d = BezierRotate::new(point, texture, transform);
            let tmp = b2d.hit_t(&r);
            println!("{:?}", tmp);
            let ans = b2d.hit(&r, tmp.unwrap());
            println!("{:?}", ans);
        };
        drawit(
            vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0), (10.0, 20.0), (0.0, 20.0)],
            // &Ray::new(Vct::new(0.001, 50.0, 0.0), Vct::new(0.0, -1.0, 0.0).norm()),
            // &Ray::new(Vct::new(5.0, 5.0, 50.0), Vct::new(0.0, 0.0, -1.0).norm()),
            // &Ray::new(Vct::new(15.0, 20.0, 0.0), Vct::new(-1.0, -1.0, 0.0).norm()),
            //&Ray::new(Vct::new(0.0, 10.0, 50.0), Vct::new(0.0, 0.0, -1.0).norm()),
        );
    }
}
