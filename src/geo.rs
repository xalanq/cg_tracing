use crate::{ray::Ray, vct::Vct, Flt};
use serde::{Deserialize, Serialize};
pub mod sphere;
pub use sphere::Sphere;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Texture {
    Diffuse,
    Specular,
    Refractive,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Geo {
    pub position: Vct,
    pub emission: Vct,
    pub color: Vct,
    pub texture: Texture,
}

impl Geo {
    pub fn new(position: Vct, emission: Vct, color: Vct, texture: Texture) -> Self {
        Self { position, emission, color, texture }
    }
}

pub trait Hittable: Send + Sync {
    fn get(&self) -> &Geo;
    fn hit(&self, r: &Ray) -> Option<Flt>;
}
