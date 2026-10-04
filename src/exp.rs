//! Portable SIMD exp.
//! Implementation selection is confined to these public entry points.
pub(crate) mod data;
pub(crate) mod fma;
pub(crate) mod full_range;
pub(crate) mod non_fma;

use core::simd::Simd;

/// Computes e^x for each lane.
#[inline]
pub fn exp_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::exp_f32(x)
    } else {
        non_fma::exp_f32(x)
    }
}

/// Computes e^x for each lane.
#[inline]
pub fn exp_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::exp_f64(x)
    } else {
        non_fma::exp_f64(x)
    }
}
