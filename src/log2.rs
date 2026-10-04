//! Portable SIMD log2.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::log::polynomial_fma::log2_f32;
    pub(super) use crate::log::table_fma::log2_f64;
}
mod non_fma {
    pub(super) use crate::log::table_free_non_fma::{log2_f32, log2_f64};
}

use core::simd::Simd;

/// Computes the base-two logarithm for each lane.
#[inline]
pub fn log2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::log2_f32(x)
    } else {
        non_fma::log2_f32(x)
    }
}

/// Computes the base-two logarithm for each lane.
#[inline]
pub fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::log2_f64(x)
    } else {
        non_fma::log2_f64(x)
    }
}
