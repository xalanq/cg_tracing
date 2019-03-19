use crate::{
    geo::{Geo, HitResult, HitTemp, Texture, TextureRaw},
    linalg::{Ray, Transform, Vct},
    Deserialize, Serialize, EPS,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub transform: Transform,
    pub texture: Texture,
}

impl Plane {
    pub fn new(transform: Transform, texture: Texture) -> Box<dyn Geo> {
        let mut ret = Self { transform, texture };
        ret.init();
        Box::new(ret)
    }
}

impl Geo for Plane {
    // init texture if it is a image
    fn init(&mut self) {
        if let Texture::Image(ref mut img) = self.texture {
            img.load();
        }
    }

    // calculate t, which means r.origin + r.direct * t is the intersection point
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        let n = self.transform.z();
        let d = n.dot(r.direct);
        if d.abs() > EPS {
            let t = n.dot(self.transform.pos() - r.origin) / d;
            if t > EPS {
                return Some((t, None));
            }
        }
        None
    }

    // return the hit result
    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        let pos = r.origin + r.direct * tmp.0;
        let n = self.transform.z();
        HitResult {
            pos,
            norm: if n.dot(r.direct) > 0.0 { n } else { -n },
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    let v = pos - self.transform.pos();
                    let px = self.transform.x().dot(v) * img.width_ratio;
                    let py = self.transform.y().dot(v) * img.height_ratio;
                    let col = img.image.get_repeat(px as isize, py as isize);
                    TextureRaw {
                        emission: Vct::zero(),
                        color: Vct::new(col.0, col.1, col.2),
                        material: img.material,
                    }
                }
            },
        }
    }
}
