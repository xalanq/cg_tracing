use crate::{utils::Flt, vct::Vct};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cam {
    pub origin: Vct,
    pub direct: Vct,
    pub ratio: Flt,
}

impl Cam {
    pub fn new(origin: Vct, direct: Vct, ratio: Flt) -> Cam {
        Cam { origin, direct, ratio }
    }
}
