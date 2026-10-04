//! Portable SIMD trunc.
//! Implementation selection is confined to these public entry points.
mod hardware {
    pub(super) use crate::round::hardware::{trunc_f32, trunc_f64};
}
mod software {
    pub(super) use crate::round::software::{trunc_f32, trunc_f64};
}

use core::simd::Simd;

/// Computes rounding toward zero for each lane.
#[inline]
pub fn trunc_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::trunc_f32(x)
    } else {
        software::trunc_f32(x)
    }
}

/// Computes rounding toward zero for each lane.
#[inline]
pub fn trunc_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::trunc_f64(x)
    } else {
        software::trunc_f64(x)
    }
}
