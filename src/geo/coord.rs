use crate::{
    linalg::{Mat, Vct},
    Deserialize, Serialize, EPS,
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Default)]
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
        p * 5.0 + self.p
    }
}
