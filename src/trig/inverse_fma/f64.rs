#[path = "f64/asin.rs"]
mod asin;
#[path = "f64/atan.rs"]
mod atan;
pub(crate) use asin::{acos_f64, asin_f64};
pub(crate) use atan::{atan2_f64, atan_f64};
