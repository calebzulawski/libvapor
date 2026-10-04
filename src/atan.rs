//! Portable SIMD atan.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::trig::inverse_fma::{atan_f32, atan_f64};
}
mod non_fma {
    pub(super) use crate::trig::inverse_non_fma::{atan_f32, atan_f64};
}

use core::simd::Simd;

/// Computes atan(x) for each lane.
#[inline]
pub fn atan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::atan_f32(x)
    } else {
        non_fma::atan_f32(x)
    }
}

/// Computes atan(x) for each lane.
#[inline]
pub fn atan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::atan_f64(x)
    } else {
        non_fma::atan_f64(x)
    }
}
