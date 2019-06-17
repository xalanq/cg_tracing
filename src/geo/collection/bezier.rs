use crate::{
    geo::{Geo, HitResult, HitTemp, Texture, TextureRaw},
    linalg::{Ray, Transform, Vct},
    Deserialize, Flt, Serialize,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            x = self.a[i - 1].0 * i as Flt + x * t;
            y = self.a[i - 1].1 * i as Flt + y * t;
        }
        (x, y)
    }

    pub fn ddp(&self, t: Flt) -> (Flt, Flt) {
        let (mut x, mut y) = (0.0, 0.0);
        for i in (2..=self.n).rev() {
            x = self.a[i - 2].0 * i as Flt * (i - 1) as Flt + x * t;
            y = self.a[i - 2].1 * i as Flt * (i - 1) as Flt + y * t;
        }
        (x, y)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bezier {
    pub point: Vec<(Flt, Flt)>,
    pub pivot: Flt,
    pub transform: Transform,
    pub texture: Texture,
    b2d: Bezier2D,
}

impl Bezier {
    pub fn new(
        point: Vec<(Flt, Flt)>,
        pivot: Flt,
        transform: Transform,
        texture: Texture,
    ) -> Box<dyn Geo> {
        let b2d = Bezier2D::new(&point);
        let ret = Self { point, pivot, transform, texture, b2d };
        Box::new(ret)
    }
}

impl Geo for Bezier {
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        None
    }

    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        HitResult {
            pos: r.origin + r.direct * tmp.0,
            norm: Vct::zero(),
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    TextureRaw { emission: Vct::zero(), color: Vct::zero(), material: img.material }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier2d() {
        let point = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0)];
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
    }
}
