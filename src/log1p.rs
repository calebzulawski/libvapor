//! Portable SIMD log1p.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::log::one_plus_fma::{log1p_f32, log1p_f64};
}
mod non_fma {
    pub(super) use crate::log::one_plus_non_fma::{log1p_f32, log1p_f64};
}

use core::simd::Simd;

/// Computes ln(1+x) for each lane.
#[inline]
pub fn log1p_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::log1p_f32(x)
    } else {
        non_fma::log1p_f32(x)
    }
}

/// Computes ln(1+x) for each lane.
#[inline]
pub fn log1p_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::log1p_f64(x)
    } else {
        non_fma::log1p_f64(x)
    }
}
