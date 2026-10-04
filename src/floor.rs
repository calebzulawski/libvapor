//! Portable SIMD floor.
//! Implementation selection is confined to these public entry points.
mod hardware {
    pub(super) use crate::round::hardware::{floor_f32, floor_f64};
}
mod software {
    pub(super) use crate::round::software::{floor_f32, floor_f64};
}

use core::simd::Simd;

/// Computes rounding toward negative infinity for each lane.
#[inline]
pub fn floor_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::floor_f32(x)
    } else {
        software::floor_f32(x)
    }
}

/// Computes rounding toward negative infinity for each lane.
#[inline]
pub fn floor_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::floor_f64(x)
    } else {
        software::floor_f64(x)
    }
}
