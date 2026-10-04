//! Portable SIMD exp2.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::exp::fma::{exp2_f32, exp2_f64};
}
mod non_fma {
    pub(super) use crate::exp::non_fma::{exp2_f32, exp2_f64};
}

use core::simd::Simd;

/// Computes 2^x for each lane.
#[inline]
pub fn exp2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::exp2_f32(x)
    } else {
        non_fma::exp2_f32(x)
    }
}

/// Computes 2^x for each lane.
#[inline]
pub fn exp2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::exp2_f64(x)
    } else {
        non_fma::exp2_f64(x)
    }
}
