#[path = "sin_cos_fma/f32.rs"]
mod f32;
#[path = "sin_cos_fma/f64.rs"]
mod f64;

pub(crate) use f32::*;
pub(crate) use f64::*;
