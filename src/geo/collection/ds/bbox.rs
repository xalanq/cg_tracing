use crate::{linalg::Vct, Flt};

#[derive(Clone, Debug, Default)]
pub struct BBox {
    pub min: Vct,
    pub max: Vct,
}

impl BBox {
    pub fn hit(&self, origin: &Vct, inv_direct: &Vct) -> Option<(Flt, Flt)> {
        let a = (self.min - *origin) * *inv_direct;
        let b = (self.max - *origin) * *inv_direct;
        let min = a.min(b);
        let max = a.max(b);
        let t_min = min.x.max(min.y).max(min.z).max(0.0);
        let t_max = max.x.min(max.y).min(max.z);
        if t_min <= t_max {
            Some((t_min, t_max))
        } else {
            None
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn fast_hit(
        &self,
        origin: &Vct,
        inv_direct: &Vct,
        neg_index: &[bool; 3],
    ) -> Option<(Flt, Flt)> {
        macro_rules! a { ($i:expr) => { if neg_index[$i] { self.max } else { self.min } }; }
        macro_rules! b { ($i:expr) => { if neg_index[$i] { self.min } else { self.max } }; }
        let mut tmin = (a!(0).x - origin.x) * inv_direct.x;
        let mut tmax = (b!(0).x - origin.x) * inv_direct.x;
        let tymin = (a!(1).y - origin.y) * inv_direct.y;
        let tymax = (b!(1).y - origin.y) * inv_direct.y;
        if tmin < 0.0 {
            tmin = 0.0;
        }
        if tmin > tymax || tymin > tmax {
            return None;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }
        let tzmin = (a!(2).z - origin.z) * inv_direct.z;
        let tzmax = (b!(2).z - origin.z) * inv_direct.z;
        if tmin > tzmax || tzmin > tmax {
            return None;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }
        if tmin <= tmax {
            return Some((tmin, tmax));
        }
        None
    }
}
