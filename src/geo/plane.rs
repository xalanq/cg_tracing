use crate::geo::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Texture {
    Raw { raw: TextureRaw },
    Image { image: TextureImage, x: Vct, y: Vct, width_ratio: Flt, height_ratio: Flt },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub p: Vct,     // any point at plane (but it's the left-bottom point of texture image)
    pub n: Vct,     // normal vector of plane
    pub t: Texture, // texture info
}

impl Plane {
    pub fn new(p: Vct, n: Vct, t: Texture) -> Box<dyn Geo> {
        let mut ret = Self { p, n, t };
        ret.init();
        Box::new(ret)
    }
}

impl Geo for Plane {
    // init texture if it is a image
    fn init(&mut self) {
        if let Texture::Image { ref mut image, ref mut x, ref mut y, width_ratio, height_ratio } =
            self.t
        {
            *x /= x.len();
            *y /= y.len();
            assert!(x.dot(y).abs() < EPS);
            assert!(width_ratio > 0.0);
            assert!(height_ratio > 0.0);
            image.load();
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

    // return the hit result
    fn hit(&self, r: &Ray, t: Flt) -> HitResult {
        let pos = r.origin + r.direct * t;
        HitResult {
            pos,
            norm: if self.n.dot(&r.direct) > 0.0 { self.n } else { -self.n },
            texture: match self.t {
                Texture::Raw { ref raw } => *raw,
                Texture::Image { ref image, ref x, ref y, ref width_ratio, ref height_ratio } => {
                    let v = pos - self.p;
                    let px = x.dot(&v) / width_ratio;
                    let py = y.dot(&v) / height_ratio;
                    let col = image.pic.get(px as isize, py as isize);
                    TextureRaw {
                        emission: Vct::zero(),
                        color: Vct::new(
                            col.0 as Flt / 255.0,
                            col.1 as Flt / 255.0,
                            col.2 as Flt / 255.0,
                        ),
                        material: image.material,
                    }
                }
            },
        }
    }
}
