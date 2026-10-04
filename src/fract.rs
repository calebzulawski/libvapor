//! Portable SIMD fractional parts.
//! Implementation selection is confined to these public entry points.
mod hardware {
    pub(super) use crate::round::hardware::{fract_f32, fract_f64};
}
mod software {
    pub(super) use crate::round::software::{fract_f32, fract_f64};
}

use core::simd::Simd;
/// Computes the fractional part of each lane.
#[inline]
pub fn fract_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::fract_f32(x)
    } else {
        software::fract_f32(x)
    }
}
/// Computes the fractional part of each lane.
#[inline]
pub fn fract_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::fract_f64(x)
    } else {
        software::fract_f64(x)
    }
}
