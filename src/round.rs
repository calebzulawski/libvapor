//! Portable SIMD round.
//! Implementation selection is confined to these public entry points.
pub(crate) mod hardware;
pub(crate) mod software;

use core::simd::Simd;

/// Computes rounding to nearest with ties away from zero for each lane.
#[inline]
pub fn round_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::round_f32(x)
    } else {
        software::round_f32(x)
    }
}

/// Computes rounding to nearest with ties away from zero for each lane.
#[inline]
pub fn round_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        hardware::round_f64(x)
    } else {
        software::round_f64(x)
    }
}
