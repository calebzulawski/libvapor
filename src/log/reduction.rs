#[path = "reduction/f32.rs"]
mod f32;
#[path = "reduction/f64.rs"]
mod f64;

pub(crate) use f32::*;
pub(crate) use f64::*;
