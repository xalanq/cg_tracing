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
        if t_min < t_max {
            Some((t_min, t_max))
        } else {
            None
        }
    }
}
