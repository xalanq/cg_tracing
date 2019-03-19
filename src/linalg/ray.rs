use crate::{linalg::Vct, Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vct,
    pub direct: Vct,
}

impl Ray {
    pub fn new(origin: Vct, direct: Vct) -> Self {
        Self { origin, direct }
    }
}
