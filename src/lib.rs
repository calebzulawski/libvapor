//! Portable SIMD implementations of basic mathematical functions.
//!
//! Arithmetic assumes round-to-nearest, ties-to-even. Transcendental functions
//! target at most 4 ULP; rounding, square root, and fused multiply-add target
//! correctly rounded results. Tests cover signed zeros and exceptional values;
//! floating-point exception flags and NaN payloads are not specified.
#![feature(portable_simd)]

mod trig;

mod trunc;
pub use trunc::{trunc_f32, trunc_f64};

mod fract;
pub use fract::{fract_f32, fract_f64};

mod floor;
pub use floor::{floor_f32, floor_f64};

mod ceil;
pub use ceil::{ceil_f32, ceil_f64};

mod round;
pub use round::{round_f32, round_f64};

mod sqrt;
pub use sqrt::{sqrt_f32, sqrt_f64};

mod fma;
pub use fma::fma_f32;

mod exp;
pub use exp::{exp_f32, exp_f64};

mod exp2;
pub use exp2::{exp2_f32, exp2_f64};

mod log;
pub use log::{log_f32, log_f64};

mod log2;
pub use log2::{log2_f32, log2_f64};

mod log10;
pub use log10::{log10_f32, log10_f64};

mod log1p;
pub use log1p::{log1p_f32, log1p_f64};

mod sin;
pub use sin::{sin_f32, sin_f64};

mod cos;
pub use cos::{cos_f32, cos_f64};

mod sincos;
pub use sincos::{sincos_f32, sincos_f64};

mod tan;
pub use tan::{tan_f32, tan_f64};

mod atan;
pub use atan::{atan_f32, atan_f64};

mod atan2;
pub use atan2::{atan2_f32, atan2_f64};

mod asin;
pub use asin::{asin_f32, asin_f64};

mod acos;
pub use acos::{acos_f32, acos_f64};
