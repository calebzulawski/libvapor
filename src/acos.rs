//! Portable SIMD acos.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::trig::inverse_fma::{acos_f32, acos_f64};
}
mod non_fma {
    pub(super) use crate::trig::inverse_non_fma::{acos_f32, acos_f64};
}

use core::simd::Simd;

/// Computes acos(x) for each lane.
#[inline]
pub fn acos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::acos_f32(x)
    } else {
        non_fma::acos_f32(x)
    }
}

/// Computes acos(x) for each lane.
#[inline]
pub fn acos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::acos_f64(x)
    } else {
        non_fma::acos_f64(x)
    }
}
