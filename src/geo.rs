use crate::{
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
pub enum Texture {
    Diffuse,
    Specular,
    Refractive,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Geo {
    pub emission: Vct,
    pub color: Vct,
    pub texture: Texture,
}

impl Geo {
    pub fn new(emission: Vct, color: Vct, texture: Texture) -> Self {
        Self { emission, color, texture }
    }
}

pub trait Hittable: Send + Sync {
    fn hit_t(&self, r: &Ray) -> Option<Flt>;
    fn hit(&self, r: &Ray, t: Flt) -> (&Geo, Vct, Vct);
}
