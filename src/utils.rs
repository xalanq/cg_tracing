pub use crate::Flt;

pub const EPS: Flt = 1e-4;

pub fn clamp(x: Flt) -> Flt {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

pub fn to_byte(x: Flt) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}
