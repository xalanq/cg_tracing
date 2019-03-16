use crate::{
    mat::Mat,
    pic::Pic,
    ray::Ray,
    utils::{Flt, EPS},
    vct::Vct,
};
use serde::{Deserialize, Serialize};
use std::default::Default;
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

impl TextureRaw {
    pub fn new(emission: Vct, color: Vct, material: Material) -> Self {
        Self { emission, color, material }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureImage {
    pub filename: String,
    pub material: Material,

    #[serde(skip_serializing, skip_deserializing)]
    pub pic: Pic,
}

impl TextureImage {
    pub fn new(filename: String, material: Material) -> Self {
        Self { filename, material, pic: Pic::default() }
    }
}

use image::GenericImageView;

impl TextureImage {
    pub fn load(&mut self) {
        let img = image::open(&self.filename).expect(&format!("Cannot open {}", self.filename));
        let (w, h) = (img.width(), img.height());
        self.pic.w = w as usize;
        self.pic.h = h as usize;
        self.pic.c = Vec::with_capacity(self.pic.w * self.pic.h);
        for (_, _, p) in img.pixels() {
            self.pic.c.push((p.data[0], p.data[1], p.data[2], p.data[3]));
        }
    }
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
