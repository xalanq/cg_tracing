use crate::{geo::*, ray::*, utils::EPS, Flt};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub p: Vct, // any point at plane
    pub n: Vct, // normal vector of plane
    pub g: Geo, // geometric info
}

impl Plane {
    pub fn new(p: Vct, n: Vct, g: Geo) -> Box<dyn Hittable> {
        Box::new(Self { p, n, g })
    }

    // just copy that func when you custom your object
    pub fn from_json(v: Value) -> Box<dyn Hittable> {
        Box::new(serde_json::from_value::<Self>(v).expect("Invalid Plane"))
    }
}

impl Hittable for Plane {
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
    fn hit(&self, r: &Ray, t: Flt) -> (&Geo, Vct, Vct) {
        (
            &self.g,
            r.origin + r.direct * t,
            if self.n.dot(&r.direct) > 0.0 { self.n } else { -self.n },
        )
    }
}
