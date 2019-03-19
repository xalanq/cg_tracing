use crate::{
    geo::{Coord, Geo, HitResult, HitTemp, Texture, TextureRaw},
    linalg::{Ray, Vct},
    Deserialize, Serialize, EPS,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub coord: Coord,
    pub texture: Texture,
}

impl Plane {
    pub fn new(coord: Coord, texture: Texture) -> Box<dyn Geo> {
        let mut ret = Self { coord, texture };
        ret.init();
        Box::new(ret)
    }
}

impl Geo for Plane {
    // init texture if it is a image
    fn init(&mut self) {
        self.coord.norm();
        if let Texture::Image(ref mut img) = self.texture {
            img.load();
        }
    }

    // calculate t, which means r.origin + r.direct * t is the intersection point
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        let d = self.coord.z.dot(r.direct);
        if d.abs() > EPS {
            let t = self.coord.z.dot(self.coord.p - r.origin) / d;
            if t > EPS {
                return Some((t, None));
            }
        }
        None
    }

    // return the hit result
    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        let pos = r.origin + r.direct * tmp.0;
        let n = self.coord.z;
        HitResult {
            pos,
            norm: if n.dot(r.direct) > 0.0 { n } else { -n },
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    let v = pos - self.coord.p;
                    let px = self.coord.x.dot(v) * img.width_ratio;
                    let py = self.coord.y.dot(v) * img.height_ratio;
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
