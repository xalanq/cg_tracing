use crate::geo::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub c: Vct,     // center
    pub r: Flt,     // radius
    pub t: Texture, // texture info
}

impl Sphere {
    pub fn new(c: Vct, r: Flt, t: Texture) -> Box<dyn Geo> {
        Box::new(Self { c, r, t })
    }
}

impl Geo for Sphere {
    fn hit_t(&self, r: &Ray) -> Option<Flt> {
        let op = self.c - r.origin;
        let b = op.dot(&r.direct);
        let det = b * b - op.len2() + self.r * self.r;
        if det < 0.0 {
            return None;
        }
        let det = det.sqrt();
        let t = b - det;
        if t > EPS {
            return Some(t);
        }
        let t = b + det;
        if t > EPS {
            return Some(t);
        }
        None
    }

    fn hit(&self, r: &Ray, t: Flt) -> HitResult {
        let pos = r.origin + r.direct * t;
        let norm = (pos - self.c).norm();
        HitResult {
            pos,
            norm,
            texture: match self.t {
                Texture::Raw(ref tx) => *tx,
                Texture::Image(ref tx) => {
                    TextureRaw { emission: Vct::zero(), color: Vct::zero(), material: tx.material }
                }
            },
        }
    }
}
