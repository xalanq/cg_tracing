use crate::{linalg::Vct, Flt};
use std::default::Default;
use std::ops::{Mul, Rem};

/*
   y
   |
   |
   |
   o--------x
  /
 /
z
*/

macro_rules! df {
    () => {
        Default::default()
    };
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Mat {
    pub m00: Flt, pub m01: Flt, pub m02: Flt, pub m03: Flt,
    pub m10: Flt, pub m11: Flt, pub m12: Flt, pub m13: Flt,
    pub m20: Flt, pub m21: Flt, pub m22: Flt, pub m23: Flt,
    pub m30: Flt, pub m31: Flt, pub m32: Flt, pub m33: Flt,
}

impl Mat {
    pub fn identity() -> Self {
        Self { m00: 1.0, m11: 1.0, m22: 1.0, m33: 1.0, ..df!() }
    }

    pub fn scale(x: Flt, y: Flt, z: Flt) -> Self {
        Self { m00: x, m11: y, m22: z, m33: 1.0, ..df!() }
    }

    pub fn shift(x: Flt, y: Flt, z: Flt) -> Self {
        Self { m00: 1.0, m11: 1.0, m22: 1.0, m33: 1.0, m03: x, m13: y, m23: z, ..df!() }
    }

    pub fn rot(axis: &str, radian: Flt) -> Self {
        let (sin, cos) = (radian.sin(), radian.cos());
        match axis {
            "x" => Self { m00: 1.0, m11: cos, m12: -sin, m21: sin, m22: cos, m33: 1.0, ..df!() },
            "y" => Self { m00: cos, m02: sin, m11: 1.0, m20: -sin, m22: cos, m33: 1.0, ..df!() },
            "z" => Self { m00: cos, m01: -sin, m10: sin, m11: cos, m22: 1.0, m33: 1.0, ..df!() },
            _ => panic!("Invalid axis"),
        }
    }

    pub fn rot_degree(axis: &str, degree: Flt) -> Self {
        Self::rot(axis, degree.to_radians())
    }

    // axis: p + tv
    pub fn rot_line(p: Vct, v: Vct, radian: Flt) -> Self {
        let a = (v.dot(Vct::new(1.0, 0.0, 0.0)) / v.len()).acos();
        let b = (v.dot(Vct::new(0.0, 1.0, 0.0)) / v.len()).acos();
        Self::shift(p.x, p.y, p.z)
            * Self::rot("x", -a)
            * Self::rot("y", -b)
            * Self::rot("z", radian)
            * Self::rot("y", b)
            * Self::rot("x", a)
            * Self::shift(-p.x, -p.y, -p.z)
    }

    pub fn rot_line_degree(p: Vct, v: Vct, degree: Flt) -> Self {
        Self::rot_line(p, v, degree.to_radians())
    }
}

impl Mul<Mat> for Mat {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Mat {
            m00: self.m00 * rhs.m00 + self.m01 * rhs.m10 + self.m02 * rhs.m20 + self.m03 * rhs.m30,
            m01: self.m00 * rhs.m01 + self.m01 * rhs.m11 + self.m02 * rhs.m21 + self.m03 * rhs.m31,
            m02: self.m00 * rhs.m02 + self.m01 * rhs.m12 + self.m02 * rhs.m22 + self.m03 * rhs.m32,
            m03: self.m00 * rhs.m03 + self.m01 * rhs.m13 + self.m02 * rhs.m23 + self.m03 * rhs.m33,
            m10: self.m10 * rhs.m00 + self.m11 * rhs.m10 + self.m12 * rhs.m20 + self.m13 * rhs.m30,
            m11: self.m10 * rhs.m01 + self.m11 * rhs.m11 + self.m12 * rhs.m21 + self.m13 * rhs.m31,
            m12: self.m10 * rhs.m02 + self.m11 * rhs.m12 + self.m12 * rhs.m22 + self.m13 * rhs.m32,
            m13: self.m10 * rhs.m03 + self.m11 * rhs.m13 + self.m12 * rhs.m23 + self.m13 * rhs.m33,
            m20: self.m20 * rhs.m00 + self.m21 * rhs.m10 + self.m22 * rhs.m20 + self.m23 * rhs.m30,
            m21: self.m20 * rhs.m01 + self.m21 * rhs.m11 + self.m22 * rhs.m21 + self.m23 * rhs.m31,
            m22: self.m20 * rhs.m02 + self.m21 * rhs.m12 + self.m22 * rhs.m22 + self.m23 * rhs.m32,
            m23: self.m20 * rhs.m03 + self.m21 * rhs.m13 + self.m22 * rhs.m23 + self.m23 * rhs.m33,
            m30: self.m30 * rhs.m00 + self.m31 * rhs.m10 + self.m32 * rhs.m20 + self.m33 * rhs.m30,
            m31: self.m30 * rhs.m01 + self.m31 * rhs.m11 + self.m32 * rhs.m21 + self.m33 * rhs.m31,
            m32: self.m30 * rhs.m02 + self.m31 * rhs.m12 + self.m32 * rhs.m22 + self.m33 * rhs.m32,
            m33: self.m30 * rhs.m03 + self.m31 * rhs.m13 + self.m32 * rhs.m23 + self.m33 * rhs.m33,
        }
    }
}

// (x, y, z, 1)
impl Mul<Vct> for Mat {
    type Output = Vct;
    fn mul(self, rhs: Vct) -> Vct {
        Vct {
            x: self.m00 * rhs.x + self.m01 * rhs.y + self.m02 * rhs.z + self.m03,
            y: self.m10 * rhs.x + self.m11 * rhs.y + self.m12 * rhs.z + self.m13,
            z: self.m20 * rhs.x + self.m21 * rhs.y + self.m22 * rhs.z + self.m23,
        }
    }
}

// (x, y, z, 0)
impl Rem<Vct> for Mat {
    type Output = Vct;
    fn rem(self, rhs: Vct) -> Vct {
        Vct {
            x: self.m00 * rhs.x + self.m01 * rhs.y + self.m02 * rhs.z,
            y: self.m10 * rhs.x + self.m11 * rhs.y + self.m12 * rhs.z,
            z: self.m20 * rhs.x + self.m21 * rhs.y + self.m22 * rhs.z,
        }
    }
}
