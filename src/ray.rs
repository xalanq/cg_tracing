use crate::vct::Vct;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray {
    pub origin: Vct,
    pub direct: Vct,
}

impl Ray {
    pub fn new(origin: Vct, direct: Vct) -> Ray {
        Ray { origin, direct }
    }
}
