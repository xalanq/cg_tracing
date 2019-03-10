use crate::{geo::*, ray::*, utils::EPS, Flt};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub c: Vct, // center
    pub r: Flt, // radius
    pub g: Geo, // geometric info
}

impl Sphere {
    pub fn new(c: Vct, r: Flt, g: Geo) -> Box<dyn Hittable> {
        Box::new(Self { c, r, g })
    }
}

impl Hittable for Sphere {
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

    fn hit(&self, r: &Ray, t: Flt) -> (&Geo, Vct, Vct) {
        let pos = r.origin + r.direct * t;
        let norm = (pos - self.c).norm();
        return (&self.g, pos, norm);
    }
}
