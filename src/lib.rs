pub mod geo;
pub mod linalg;
#[macro_use]
pub mod macros;
pub mod prelude;
pub mod scene;
pub mod utils;

pub use serde::{Deserialize, Serialize};

pub type Flt = f64;

pub const EPS: Flt = 1e-6;
pub const PI: Flt = std::f64::consts::PI as Flt;
