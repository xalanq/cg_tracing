use crate::utils::Flt;
use crate::vct::Vct;
use std::default::Default;
use std::ops::Mul;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Default, Copy, Clone)]
pub struct Mat {
    pub m00: Flt, pub m01: Flt, pub m02: Flt, pub m03: Flt,
    pub m10: Flt, pub m11: Flt, pub m12: Flt, pub m13: Flt,
    pub m20: Flt, pub m21: Flt, pub m22: Flt, pub m23: Flt,
    pub m30: Flt, pub m31: Flt, pub m32: Flt, pub m33: Flt,
}

impl Mat {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn world_to_object(x: Vct, y: Vct, z: Vct, p: Vct) -> Vct {
        let m = Self {
            m00: x.x, m10: y.x, m20: z.x,
            m01: x.y, m11: y.y, m21: z.y,
            m02: x.z, m12: y.z, m22: z.z,
            m33: 1.0, ..Default::default()
        };
        m * p
    }
}

impl Mul<Mat> for Mat {
    type Output = Mat;
    fn mul(self, rhs: Mat) -> Mat {
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
