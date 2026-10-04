#[path = "software/f32.rs"]
mod f32;
#[path = "software/f64.rs"]
mod f64;
#[path = "software/table.rs"]
mod table;

pub(crate) use f32::sqrt_f32;
pub(crate) use f64::sqrt_f64;
