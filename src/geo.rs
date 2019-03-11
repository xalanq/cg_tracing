use crate::{
    pic::Pic,
    ray::Ray,
    utils::{Flt, EPS},
    vct::Vct,
};
use serde::{Deserialize, Serialize};
pub mod sphere;
pub use sphere::Sphere;
pub mod plane;
pub use plane::Plane;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Material {
    Diffuse,
    Specular,
    Refractive,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TextureRaw {
    pub emission: Vct,
    pub color: Vct,
    pub material: Material,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureImage {
    pub filename: String,
    pub material: Material,
    pub up: Vct,
    pub right: Vct,

    #[serde(skip_serializing, skip_deserializing)]
    pub pic: Pic,
}

impl TextureImage {
    pub fn load(&mut self) {}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Texture {
    Raw(TextureRaw),
    Image(TextureImage),
}

#[derive(Copy, Clone, Debug)]
pub struct HitResult {
    pub pos: Vct,
    pub norm: Vct,
    pub texture: TextureRaw,
}

pub trait Geo: Send + Sync {
    fn init(&mut self) {} // use it in from_json
    fn hit_t(&self, r: &Ray) -> Option<Flt>;
    fn hit(&self, r: &Ray, t: Flt) -> HitResult;
}
