use crate::{linalg::Vct, Deserialize, Flt, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub origin: Vct,
    pub direct: Vct,
    pub ratio: Flt,
}

impl Camera {
    pub fn new(origin: Vct, direct: Vct, ratio: Flt) -> Self {
        Self { origin, direct, ratio }
    }
}
