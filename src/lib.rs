//! Portable SIMD implementations of basic mathematical functions.
//!
//! Arithmetic assumes round-to-nearest, ties-to-even. Transcendental functions
//! target at most 4 ULP; rounding, square root, and fused multiply-add target
//! correctly rounded results. Tests cover signed zeros and exceptional values;
//! floating-point exception flags and NaN payloads are not specified.
#![feature(portable_simd)]

mod backend;
mod table;

mod round;
pub use round::{
    ceil_f32, ceil_f64, floor_f32, floor_f64, fract_f32, fract_f64, round_f32, round_f64,
    trunc_f32, trunc_f64,
};

mod sqrt;
pub use sqrt::{sqrt_f32, sqrt_f64};

mod fma;
pub use fma::{fma_f32, fma_f64};

mod exp;
pub use exp::{exp2_f32, exp2_f64, exp_f32, exp_f64};

mod log;
pub use log::{log10_f32, log10_f64, log1p_f32, log1p_f64, log2_f32, log2_f64, log_f32, log_f64};

mod trig;
pub use trig::{
    acos_f32, acos_f64, asin_f32, asin_f64, atan2_f32, atan2_f64, atan_f32, atan_f64, cos_f32,
    cos_f64, sin_f32, sin_f64, sincos_f32, sincos_f64, tan_f32, tan_f64,
};
