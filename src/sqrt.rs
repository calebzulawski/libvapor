//! Portable SIMD sqrt.
//! Implementation selection is confined to these public entry points.
mod hardware;
mod software;

use core::simd::Simd;

/// Computes the correctly rounded square root for each lane.
#[inline]
pub fn sqrt_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "sse2",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::sqrt_f32(x)
    } else {
        software::sqrt_f32(x)
    }
}

/// Computes the correctly rounded square root for each lane.
#[inline]
pub fn sqrt_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "sse2",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::sqrt_f64(x)
    } else {
        software::sqrt_f64(x)
    }
}
