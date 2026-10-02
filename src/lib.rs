#![feature(portable_simd)]

mod round;
pub use round::*;

mod sqrt;
pub use sqrt::*;

mod fma;
pub use fma::*;

mod exp;
pub use exp::*;

mod trig;
pub use trig::*;

mod log;
pub use log::*;
