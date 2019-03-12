use crate::geo::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Texture {
    Raw { raw: TextureRaw },
    Image { image: TextureImage, x: Vct, y: Vct, z: Vct },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub c: Vct,     // center
    pub r: Flt,     // radius
    pub t: Texture, // texture info
}

impl Sphere {
    pub fn new(c: Vct, r: Flt, t: Texture) -> Box<dyn Geo> {
        let mut ret = Self { c, r, t };
        ret.init();
        Box::new(ret)
    }
}

impl Geo for Sphere {
    fn init(&mut self) {
        if let Texture::Image { ref mut image, ref mut x, ref mut y, ref mut z } = self.t {
            *x /= x.len();
            *y /= y.len();
            *z /= z.len();
            assert_eq!(x.dot(y).abs() < EPS, true);
            assert_eq!(x.dot(z).abs() < EPS, true);
            assert_eq!(y.dot(z).abs() < EPS, true);
            image.load();
        }
    }

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
                Texture::Raw { ref raw } => *raw,
                Texture::Image { ref image, ref x, ref y, ref z } => {
                    let p = Mat::world_to_object(*x, *y, *z, pos - self.c).norm();
                    let px = (p.y * 0.5 + 0.5) * image.pic.w as Flt;
                    let py = (p.z * 0.5 + 0.5) * image.pic.h as Flt;
                    let col = image.pic.get(px as isize, py as isize);
                    TextureRaw {
                        emission: Vct::zero(),
                        color: Vct::new(
                            col.0 as Flt / 255.0,
                            col.1 as Flt / 255.0,
                            col.2 as Flt / 255.0,
                        ),
                        material: if col.3 > 0 { Material::Diffuse } else { image.material },
                    }
                }
            },
        }
    }
}
