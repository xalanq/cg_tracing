use crate::{ray::Ray, vct::Vct, Flt};
pub mod sphere;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Texture {
    Diffuse,
    Specular,
    Refractive,
}

#[derive(Copy, Clone, PartialEq, Debug)]
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

pub trait Hittable {
    fn get(&self) -> &Geo;
    fn hit(&self, r: &Ray) -> Option<Flt>;
}
