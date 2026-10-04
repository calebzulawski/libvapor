//! Portable SIMD cos.
//! Implementation selection is confined to these public entry points.
mod fma {
    use core::simd::Simd;
    #[inline]
    pub(super) fn cos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
        crate::trig::sin_cos_fma::sin_cos_f32::<N, true>(x)
    }
    #[inline]
    pub(super) fn cos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
        crate::trig::sin_cos_fma::sin_cos_f64::<N, true>(x)
    }
}
mod non_fma {
    use core::simd::Simd;
    #[inline]
    pub(super) fn cos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
        crate::trig::sin_cos_non_fma_fast::sin_cos_f32::<N, true>(x)
    }
    #[inline]
    pub(super) fn cos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
        crate::trig::sin_cos_non_fma::full_range_sincos_f64(x).1
    }
}

use core::simd::Simd;

/// Computes cos(x) for each lane.
#[inline]
pub fn cos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::cos_f32(x)
    } else {
        non_fma::cos_f32(x)
    }
}

/// Computes cos(x) for each lane.
#[inline]
pub fn cos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::cos_f64(x)
    } else {
        non_fma::cos_f64(x)
    }
}
