//! Portable SIMD log.
//! Implementation selection is confined to these public entry points.
pub(crate) mod data;
pub(crate) mod one_plus_fma;
pub(crate) mod one_plus_non_fma;
pub(crate) mod polynomial_fma;
pub(crate) mod reduction;
pub(crate) mod series;
pub(crate) mod table_fma;
pub(crate) mod table_free_non_fma;
mod fma {
    pub(super) use super::polynomial_fma::log_f32;
    pub(super) use super::table_fma::log_f64;
}
mod non_fma {
    pub(super) use super::table_free_non_fma::{log_f32, log_f64};
}

use core::simd::Simd;

/// Computes the natural logarithm for each lane.
#[inline]
pub fn log_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::log_f32(x)
    } else {
        non_fma::log_f32(x)
    }
}

/// Computes the natural logarithm for each lane.
#[inline]
pub fn log_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::log_f64(x)
    } else {
        non_fma::log_f64(x)
    }
}
