#[path = "sin_cos_non_fma/f32.rs"]
mod f32;
#[path = "sin_cos_non_fma/f64.rs"]
mod f64;
#[path = "sin_cos_non_fma/finish.rs"]
mod finish;

pub(crate) use f32::full_range_sincos_f32;
pub(crate) use f64::full_range_sincos_f64;
