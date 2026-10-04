#[path = "polynomial_fma/log.rs"]
mod log;
#[path = "polynomial_fma/log10.rs"]
mod log10;
#[path = "polynomial_fma/log2.rs"]
mod log2;

pub(crate) use log::log_f32;
pub(crate) use log10::log10_f32;
pub(crate) use log2::log2_f32;
