#[path = "f32/log.rs"]
mod log;
#[path = "f32/log10.rs"]
mod log10;
#[path = "f32/log2.rs"]
mod log2;
pub(crate) use log::log_f32;
pub(crate) use log10::log10_f32;
pub(crate) use log2::log2_f32;
