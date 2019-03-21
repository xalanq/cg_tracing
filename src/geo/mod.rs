pub mod collection;
pub mod texture;

pub use texture::{Material, Texture, TextureImage, TextureRaw};

use crate::{
    linalg::{Ray, Vct},
    Flt,
};

#[derive(Copy, Clone, Debug)]
pub struct HitResult {
    pub pos: Vct,
    pub norm: Vct,
    pub texture: TextureRaw,
}

pub type HitTemp = (Flt, Option<(usize, Flt, Flt)>);

pub trait Geo: Send + Sync {
    fn hit_t(&self, r: &Ray) -> Option<HitTemp>;
    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult;
}
