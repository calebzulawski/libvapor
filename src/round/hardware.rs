use core::simd::Simd;
use std::simd::StdFloat;
#[inline]
pub fn trunc_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    x.trunc()
}
#[inline]
pub fn trunc_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    x.trunc()
}
#[inline]
pub fn floor_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    x.floor()
}
#[inline]
pub fn floor_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    x.floor()
}
#[inline]
pub fn ceil_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    x.ceil()
}
#[inline]
pub fn ceil_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    x.ceil()
}
#[inline]
pub fn round_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    x.round()
}
#[inline]
pub fn round_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    x.round()
}
