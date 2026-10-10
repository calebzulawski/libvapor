//! Portable SIMD sqrt with hardware and software implementations.
mod hardware;
mod software;

use core::simd::Simd;

/// Computes the correctly rounded square root for each lane.
#[inline]
pub fn sqrt_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_sqrt_f32::<N>() {
        hardware::sqrt_f32(x)
    } else {
        software::sqrt_f32(x)
    }
}

/// Computes the correctly rounded square root for each lane.
#[inline]
pub fn sqrt_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_sqrt_f64::<N>() {
        hardware::sqrt_f64(x)
    } else {
        software::sqrt_f64(x)
    }
}

/// Computes the reciprocal square root for each lane, targeting at most 4 ULP.
/// Signed zeros give signed infinities; negative inputs give NaN.
#[inline]
pub fn rsqrt_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    Simd::splat(1.0) / sqrt_f32(x)
}

/// Computes the reciprocal square root for each lane, targeting at most 4 ULP.
/// Signed zeros give signed infinities; negative inputs give NaN.
#[inline]
pub fn rsqrt_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    Simd::splat(1.0) / sqrt_f64(x)
}
