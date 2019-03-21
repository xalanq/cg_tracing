use crate::{
    geo::{Geo, HitResult, HitTemp, Material, Texture, TextureRaw},
    linalg::{Ray, Transform, Vct},
    Deserialize, Flt, Serialize, EPS,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub radius: Flt,
    pub transform: Transform,
    pub texture: Texture,
}

impl Sphere {
    pub fn new(radius: Flt, transform: Transform, texture: Texture) -> Box<dyn Geo> {
        let ret = Self { radius, transform, texture };
        Box::new(ret)
    }
}

impl Geo for Sphere {
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        let op = self.transform.pos() - r.origin;
        let b = op.dot(r.direct);
        let det = b * b - op.len2() + self.radius * self.radius;
        if det < 0.0 {
            return None;
        }
        let det = det.sqrt();
        let t = b - det;
        if t > EPS {
            return Some((t, None));
        }
        let t = b + det;
        if t > EPS {
            return Some((t, None));
        }
        None
    }

    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        let pos = r.origin + r.direct * tmp.0;
        HitResult {
            pos,
            norm: (pos - self.transform.pos()).norm(),
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    let p = (self.transform.inv * pos).norm();
                    let px = (p.x * 0.5 + 0.5) * img.image.w as Flt;
                    let py = (p.y * 0.5 + 0.5) * img.image.h as Flt;
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
