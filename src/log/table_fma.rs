#[path = "table_fma/log.rs"]
mod log;
#[path = "table_fma/log10.rs"]
mod log10;
#[path = "table_fma/log2.rs"]
mod log2;

pub(crate) use log::log_f64;
pub(crate) use log10::log10_f64;
pub(crate) use log2::log2_f64;
