//! Portable SIMD fma with hardware and software implementations.
mod hardware;
mod software;

use core::simd::Simd;

/// Computes the correctly rounded fused multiply-add x*y+z for each lane.
#[inline]
pub fn fma_f32<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>, z: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        hardware::fma_f32(x, y, z)
    } else {
        software::fma_f32(x, y, z)
    }
}

/// Computes the correctly rounded fused multiply-add x*y+z for each lane.
#[inline]
pub fn fma_f64<const N: usize>(x: Simd<f64, N>, y: Simd<f64, N>, z: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        hardware::fma_f64(x, y, z)
    } else {
        software::fma_f64(x, y, z)
    }
}
