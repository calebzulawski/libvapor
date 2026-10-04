//! Portable SIMD tan.
//! Implementation selection is confined to these public entry points.
mod fma {
    pub(super) use crate::trig::tan_fma::{tan_f32, tan_f64};
}
mod non_fma {
    pub(super) use crate::trig::tan::{tan_f32, tan_f64};
}

use core::simd::Simd;

/// Computes tan(x) for each lane.
#[inline]
pub fn tan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::tan_f32(x)
    } else {
        non_fma::tan_f32(x)
    }
}

/// Computes tan(x) for each lane.
#[inline]
pub fn tan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::tan_f64(x)
    } else {
        non_fma::tan_f64(x)
    }
}
