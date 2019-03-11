use crate::geo::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub p: Vct,     // any point at plane (but it's the left-bottom point of texture image)
    pub n: Vct,     // normal vector of plane
    pub t: Texture, // texture info
}

impl Plane {
    pub fn new(p: Vct, n: Vct, t: Texture) -> Box<dyn Geo> {
        Box::new(Self { p, n, t })
    }
}

impl Geo for Plane {
    // init texture if it is a image
    fn init(&mut self) {
        if let Texture::Image(ref mut t) = self.t {
            t.load();
        }
    }

    // calculate t, which means r.origin + r.direct * t is the intersection point
    fn hit_t(&self, r: &Ray) -> Option<Flt> {
        let d = self.n.dot(&r.direct);
        if d.abs() > EPS {
            let t = self.n.dot(&(self.p - r.origin)) / d;
            if t > EPS {
                return Some(t);
            }
        }
        None
    }

    // return geo, hit position, normal vector
    fn hit(&self, r: &Ray, t: Flt) -> HitResult {
        let pos = r.origin + r.direct * t;
        HitResult {
            pos,
            norm: if self.n.dot(&r.direct) > 0.0 { self.n } else { -self.n },
            texture: match self.t {
                Texture::Raw(ref tx) => *tx,
                Texture::Image(ref tx) => {
                    let v = pos - self.p;
                    let y = tx.up.dot(&v) / tx.up.len();
                    let x = tx.right.dot(&v) / tx.right.len();
                    let col = tx.pic.get(x as usize, y as usize);
                    TextureRaw { emission: Vct::zero(), color: col, material: tx.material }
                }
            },
        }
    }
}
