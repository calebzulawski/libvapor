//! Portable SIMD sincos.
//! Implementation selection is confined to these public entry points.
mod fma {
    use core::simd::Simd;
    #[inline]
    pub(super) fn sincos_f32<const N: usize>(x: Simd<f32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
        crate::trig::sin_cos_pair_fma::sincos_f32(x)
    }
    #[inline]
    pub(super) fn sincos_f64<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
        crate::trig::sin_cos_pair_fma::sincos_f64(x)
    }
}
mod non_fma {
    use core::simd::Simd;
    #[inline]
    pub(super) fn sincos_f32<const N: usize>(x: Simd<f32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
        crate::trig::sin_cos_pair_non_fma::sincos_f32(x)
    }
    #[inline]
    pub(super) fn sincos_f64<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
        crate::trig::sin_cos_non_fma::full_range_sincos_f64(x)
    }
}

use core::simd::Simd;

/// Computes (sin(x), cos(x)) for each lane.
#[inline]
pub fn sincos_f32<const N: usize>(x: Simd<f32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::sincos_f32(x)
    } else {
        non_fma::sincos_f32(x)
    }
}

/// Computes (sin(x), cos(x)) for each lane.
#[inline]
pub fn sincos_f64<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    if cfg!(any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )) {
        fma::sincos_f64(x)
    } else {
        non_fma::sincos_f64(x)
    }
}
