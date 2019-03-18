use crate::geo::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub radius: Flt,
    pub coord: Coord,
    pub texture: Texture,
}

impl Sphere {
    pub fn new(radius: Flt, coord: Coord, texture: Texture) -> Box<dyn Geo> {
        let mut ret = Self { radius, coord, texture };
        ret.init();
        Box::new(ret)
    }
}

impl Geo for Sphere {
    fn init(&mut self) {
        self.coord.norm();
        if let Texture::Image(ref mut img) = self.texture {
            img.load();
        }
    }

    fn hit_t(&self, r: &Ray) -> Option<(Flt, Option<(usize, Flt, Flt)>)> {
        let op = self.coord.p - r.origin;
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

    fn hit(&self, r: &Ray, tmp: (Flt, Option<(usize, Flt, Flt)>)) -> HitResult {
        let pos = r.origin + r.direct * tmp.0;
        HitResult {
            pos,
            norm: (pos - self.coord.p).norm(),
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    let p = self.coord.to_object(pos).norm();
                    let px = (p.y * 0.5 + 0.5) * img.image.w as Flt;
                    let py = (p.z * 0.5 + 0.5) * img.image.h as Flt;
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
