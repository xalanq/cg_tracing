use crate::{
    mat::Mat,
    ray::Ray,
    texture::*,
    utils::{Flt, EPS},
    vct::Vct,
};
use serde::{Deserialize, Serialize};
pub mod sphere;
pub use sphere::Sphere;
pub mod plane;
pub use plane::Plane;
pub mod mesh;
pub use mesh::Mesh;

#[derive(Copy, Clone, Debug)]
pub struct HitResult {
    pub pos: Vct,
    pub norm: Vct,
    pub texture: TextureRaw,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Coord {
    pub p: Vct,
    pub x: Vct,
    pub y: Vct,
    pub z: Vct,
}

impl Coord {
    pub fn new(p: Vct, x: Vct, y: Vct, z: Vct) -> Self {
        Self { p, x, y, z }
    }

    pub fn norm(&mut self) {
        self.x = self.x.norm();
        self.y = self.y.norm();
        self.z = self.z.norm();
        assert!(self.x.dot(self.y).abs() < EPS);
        assert!(self.x.dot(self.z).abs() < EPS);
        assert!(self.y.dot(self.z).abs() < EPS);
    }

    pub fn to_object(&self, p: Vct) -> Vct {
        Mat::world_to_object(self.x, self.y, self.z, p - self.p)
    }

    pub fn to_world(&self, p: Vct) -> Vct {
        Mat::object_to_world(self.x, self.y, self.z, p) + self.p
    }
}

pub trait Geo: Send + Sync {
    fn init(&mut self) {} // use it in from_json
    fn hit_t(&self, r: &Ray) -> Option<(Flt, Option<(usize, Flt, Flt)>)>;
    fn hit(&self, r: &Ray, tmp: (Flt, Option<(usize, Flt, Flt)>)) -> HitResult;
}
