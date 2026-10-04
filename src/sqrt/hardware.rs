use core::simd::Simd;
use std::simd::StdFloat;
#[inline]
pub fn sqrt_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    x.sqrt()
}
#[inline]
pub fn sqrt_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    x.sqrt()
}
