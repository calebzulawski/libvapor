#[path = "f32/asin.rs"]
mod asin;
#[path = "f32/atan.rs"]
mod atan;
pub(crate) use asin::{acos_f32, asin_f32};
pub(crate) use atan::{atan2_f32, atan_f32};
