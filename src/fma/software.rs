#[path = "software/f32.rs"]
mod f32;
#[path = "software/f64.rs"]
mod f64;

pub(crate) use f32::fma_f32;
pub(crate) use f64::fma_f64;
