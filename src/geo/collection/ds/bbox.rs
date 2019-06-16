use crate::{linalg::Vct, Flt};

#[derive(Clone, Debug, Default)]
pub struct BBox {
    pub min: Vct,
    pub max: Vct,
}

impl BBox {
    pub fn hit(&self, origin: &Vct, inv_direct: &Vct) -> Option<(Flt, Flt)> {
        let neg_index = &[inv_direct.x < 0.0, inv_direct.y < 0.0, inv_direct.z < 0.0];
        self.fast_hit(origin, inv_direct, neg_index)
        /*
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
        */
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
        let mut t_min = (a!(0).x - origin.x) * inv_direct.x;
        let mut t_max = (b!(0).x - origin.x) * inv_direct.x;
        let ty_min = (a!(1).y - origin.y) * inv_direct.y;
        let ty_max = (b!(1).y - origin.y) * inv_direct.y;
        if t_min < 0.0 {
            t_min = 0.0;
        }
        if t_min > ty_max || ty_min > t_max {
            return None;
        }
        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }
        let tz_min = (a!(2).z - origin.z) * inv_direct.z;
        let tz_max = (b!(2).z - origin.z) * inv_direct.z;
        if t_min > tz_max || tz_min > t_max {
            return None;
        }
        if tz_min > t_min {
            t_min = tz_min;
        }
        if tz_max < t_max {
            t_max = tz_max;
        }
        if t_min < t_max {
            Some((t_min, t_max))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit() {
        let b = BBox { min: Vct::new(0.0, 0.0, 0.0), max: Vct::new(1.0, 1.0, 1.0) };
        let t = b.hit(&Vct::new(0.5, 0.5, 0.5), &Vct::new(1.0, 1.0 / 0.0, 1.0 / 0.0));
        assert!(!t.is_none());
        let t = t.unwrap();
        assert!((t.0 - 0.0).abs() < 1e-5);
        assert!((t.1 - 0.5).abs() < 1e-5);
        let t = b.hit(&Vct::new(-0.5, 0.5, 0.5), &Vct::new(1.0, 1.0 / 0.0, 1.0 / 0.0));
        assert!(!t.is_none());
        let t = t.unwrap();
        assert!((t.0 - 0.5).abs() < 1e-5);
        assert!((t.1 - 1.5).abs() < 1e-5);
    }
}