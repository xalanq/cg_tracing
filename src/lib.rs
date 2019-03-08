pub type Flt = f32;
const EPS: Flt = 1e-4;

pub fn clamp(x: Flt) -> Flt {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

pub fn toByte(x: Flt) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

pub mod vct;
pub mod pic;
