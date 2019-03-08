use crate::Flt;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::ops::{Div, DivAssign, Mul, MulAssign};
use std::ops::{Neg, Rem};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vct {
    pub x: Flt,
    pub y: Flt,
    pub z: Flt,
}

impl Vct {
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn new(x: Flt, y: Flt, z: Flt) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, rhs: &Self) -> Flt {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn len2(&self) -> Flt {
        self.dot(self)
    }

    pub fn len(&self) -> Flt {
        self.len2().sqrt()
    }

    pub fn norm(&self) -> Self {
        *self / self.len()
    }
}

impl fmt::Display for Vct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/* Add */

impl Add<Vct> for Vct {
    type Output = Vct;
    fn add(self, rhs: Vct) -> Vct {
        Vct::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Flt> for Vct {
    type Output = Vct;
    fn add(self, rhs: Flt) -> Vct {
        Vct::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl AddAssign<Vct> for Vct {
    fn add_assign(&mut self, rhs: Vct) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<Flt> for Vct {
    fn add_assign(&mut self, rhs: Flt) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

/* Sub */

impl Sub<Vct> for Vct {
    type Output = Vct;
    fn sub(self, rhs: Vct) -> Vct {
        Vct::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<Flt> for Vct {
    type Output = Vct;
    fn sub(self, rhs: Flt) -> Vct {
        Vct::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl SubAssign<Vct> for Vct {
    fn sub_assign(&mut self, rhs: Vct) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<Flt> for Vct {
    fn sub_assign(&mut self, rhs: Flt) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}

/* Mul */

impl Mul<Vct> for Vct {
    type Output = Vct;
    fn mul(self, rhs: Vct) -> Vct {
        Vct::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<Flt> for Vct {
    type Output = Vct;
    fn mul(self, rhs: Flt) -> Vct {
        Vct::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign<Vct> for Vct {
    fn mul_assign(&mut self, rhs: Vct) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<Flt> for Vct {
    fn mul_assign(&mut self, rhs: Flt) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

/* Div */

impl Div<Vct> for Vct {
    type Output = Vct;
    fn div(self, rhs: Vct) -> Vct {
        Vct::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<Flt> for Vct {
    type Output = Vct;
    fn div(self, rhs: Flt) -> Vct {
        Vct::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign<Vct> for Vct {
    fn div_assign(&mut self, rhs: Vct) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<Flt> for Vct {
    fn div_assign(&mut self, rhs: Flt) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

/* Neg */

impl Neg for Vct {
    type Output = Vct;
    fn neg(self) -> Vct {
        Vct::new(-self.x, -self.y, -self.z)
    }
}

/* Cross(Rem) */

impl Rem<Vct> for Vct {
    type Output = Vct;
    fn rem(self, rhs: Vct) -> Vct {
        Vct::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen() -> (Vct, Vct) {
        (Vct::new(1., 2., 3.), Vct::new(2., 3., 4.))
    }

    #[test]
    fn add() {
        let (x, y) = gen();
        let c = Vct::new(3., 5., 7.);
        assert_eq!(x + y, c);
        assert_eq!(x + 2., Vct::new(x.x + 2., x.y + 2., x.z + 2.));
        let mut x = x;
        x += y;
        assert_eq!(x, c);
    }

    #[test]
    fn sub() {
        let (x, y) = gen();
        let c = Vct::new(-1., -1., -1.);
        assert_eq!(x - y, c);
        assert_eq!(x - 2., Vct::new(x.x - 2., x.y - 2., x.z - 2.));
        let mut x = x;
        x -= y;
        assert_eq!(x, c);
    }

    #[test]
    fn mul() {
        let (x, y) = gen();
        let c = Vct::new(2., 6., 12.);
        assert_eq!(x * y, c);
        assert_eq!(x * 2., Vct::new(x.x * 2., x.y * 2., x.z * 2.));
        let mut x = x;
        x *= y;
        assert_eq!(x, c);
    }

    #[test]
    fn div() {
        let (x, y) = gen();
        let c = Vct::new(1. / 2., 2. / 3., 3. / 4.);
        assert_eq!(x / y, c);
        assert_eq!(x / 2., Vct::new(x.x / 2., x.y / 2., x.z / 2.));
        let mut x = x;
        x /= y;
        assert_eq!(x, c);
    }

    #[test]
    fn neg() {
        let (x, _) = gen();
        let c = Vct::new(-1., -2., -3.);
        assert_eq!(-x, c);
    }
}
