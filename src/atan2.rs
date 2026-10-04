//! Portable SIMD atan2.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::trig::inverse_fma::{atan2_f32, atan2_f64};
}
mod non_fma {
    pub(super) use crate::trig::inverse_non_fma::{atan2_f32, atan2_f64};
}

use core::simd::Simd;

/// Computes atan2(y,x) for each lane.
#[inline]
pub fn atan2_f32<const N: usize>(y: Simd<f32, N>, x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::atan2_f32(y, x)
    } else {
        non_fma::atan2_f32(y, x)
    }
}

/// Computes atan2(y,x) for each lane.
#[inline]
pub fn atan2_f64<const N: usize>(y: Simd<f64, N>, x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::atan2_f64(y, x)
    } else {
        non_fma::atan2_f64(y, x)
    }
}
