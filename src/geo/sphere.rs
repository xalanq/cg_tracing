use crate::{geo::*, ray::Ray, utils::EPS, Flt};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub r: Flt,
    pub g: Geo,
}

impl Sphere {
    pub fn new(r: Flt, g: Geo) -> Box<dyn Hittable> {
        Box::new(Self { r, g })
    }
}

impl Hittable for Sphere {
    fn get(&self) -> &Geo {
        &self.g
    }

    fn hit(&self, r: &Ray) -> Option<Flt> {
        let g = &self.g;
        let op = g.position - r.origin;
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
}
