//! Portable SIMD implementations of basic mathematical functions.
//!
//! Arithmetic assumes round-to-nearest, ties-to-even. Transcendental functions
//! target at most 4 ULP; rounding, square root, fused multiply-add, and remainders
//! target correctly rounded results. Tests cover signed zeros and exceptional
//! values; floating-point exception flags and NaN payloads are not specified.
#![feature(portable_simd)]

mod backend;
mod precision;
mod table;

// Match the function-family order in the benchmark and plot lists.
mod round;
#[rustfmt::skip]
pub use round::{
    trunc_f32, trunc_f64,
    floor_f32, floor_f64, ceil_f32, ceil_f64,
    round_f32, round_f64,
};

mod remainder;
pub use remainder::{fmod_f32, fmod_f64, remainder_f32, remainder_f64};

mod sqrt;
pub use sqrt::{sqrt_f32, sqrt_f64};

mod cbrt;
pub use cbrt::{cbrt_f32, cbrt_f64};

mod hypot;
pub use hypot::{hypot_f32, hypot_f64};

mod fma;
pub use fma::{fma_f32, fma_f64};

mod exp;
#[rustfmt::skip]
pub use exp::{exp_f32, exp_f64, exp2_f32, exp2_f64};

mod expm1;
pub use expm1::{expm1_f32, expm1_f64};

mod pow;
pub use pow::{pow_f32, pow_f64};

mod log;
#[rustfmt::skip]
pub use log::{
    log_f32, log_f64, log2_f32, log2_f64,
    log10_f32, log10_f64, log1p_f32, log1p_f64,
};

mod trig;
#[rustfmt::skip]
pub use trig::{
    sin_f32, sin_f64, cos_f32, cos_f64,
    sincos_f32, sincos_f64, tan_f32, tan_f64,
    asin_f32, asin_f64, acos_f32, acos_f64,
    atan_f32, atan_f64, atan2_f32, atan2_f64,
};

mod hyperbolic;
#[rustfmt::skip]
pub use hyperbolic::{
    sinh_f32, sinh_f64, cosh_f32, cosh_f64,
    tanh_f32, tanh_f64, asinh_f32, asinh_f64,
    acosh_f32, acosh_f64, atanh_f32, atanh_f64,
};

mod erf;
pub use erf::{erf_f32, erf_f64, erfc_f32, erfc_f64};

mod gamma;
pub use gamma::{lgamma_f32, lgamma_f64, tgamma_f32, tgamma_f64};
