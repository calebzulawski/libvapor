//! Portable SIMD rounding.
mod hardware;
mod software;

use core::simd::{num::SimdFloat, Simd};

use crate::backend::RoundingMode;

/// Computes rounding toward zero for each lane.
#[inline]
pub fn trunc_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_round_f32::<N>(RoundingMode::Directed) {
        hardware::trunc_f32(x)
    } else {
        software::trunc_f32(x)
    }
}

/// Computes rounding toward zero for each lane.
#[inline]
pub fn trunc_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_round_f64::<N>(RoundingMode::Directed) {
        hardware::trunc_f64(x)
    } else {
        software::trunc_f64(x)
    }
}

/// Computes rounding toward negative infinity for each lane.
#[inline]
pub fn floor_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_round_f32::<N>(RoundingMode::Directed) {
        hardware::floor_f32(x)
    } else {
        software::floor_f32(x)
    }
}

/// Computes rounding toward negative infinity for each lane.
#[inline]
pub fn floor_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_round_f64::<N>(RoundingMode::Directed) {
        hardware::floor_f64(x)
    } else {
        software::floor_f64(x)
    }
}

/// Computes rounding toward positive infinity for each lane.
#[inline]
pub fn ceil_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_round_f32::<N>(RoundingMode::Directed) {
        hardware::ceil_f32(x)
    } else {
        software::ceil_f32(x)
    }
}

/// Computes rounding toward positive infinity for each lane.
#[inline]
pub fn ceil_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_round_f64::<N>(RoundingMode::Directed) {
        hardware::ceil_f64(x)
    } else {
        software::ceil_f64(x)
    }
}

/// Computes rounding to nearest with ties away from zero for each lane.
#[inline]
pub fn round_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_round_f32::<N>(RoundingMode::TiesAway) {
        hardware::round_f32(x)
    } else {
        software::round_f32(x)
    }
}

/// Computes rounding to nearest with ties away from zero for each lane.
#[inline]
pub fn round_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_round_f64::<N>(RoundingMode::TiesAway) {
        hardware::round_f64(x)
    } else {
        software::round_f64(x)
    }
}

/// Computes rounding to nearest with ties to even for each lane.
#[inline]
pub fn roundeven_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_round_f32::<N>(RoundingMode::TiesEven) {
        hardware::roundeven_f32(x)
    } else {
        software::roundeven_f32(x)
    }
}

/// Computes rounding to nearest with ties to even for each lane.
#[inline]
pub fn roundeven_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_round_f64::<N>(RoundingMode::TiesEven) {
        hardware::roundeven_f64(x)
    } else {
        software::roundeven_f64(x)
    }
}

/// Rounds each lane to the nearest `i32`, with ties away from zero.
///
/// Results outside the integer range (including infinities) saturate to
/// `i32::MIN` or `i32::MAX`; NaNs produce zero.
#[inline]
pub fn lround_f32<const N: usize>(x: Simd<f32, N>) -> Simd<i32, N> {
    round_f32(x).cast()
}

/// Rounds each lane to the nearest `i32`, with ties away from zero.
///
/// Results outside the integer range (including infinities) saturate to
/// `i32::MIN` or `i32::MAX`; NaNs produce zero.
#[inline]
pub fn lround_f64<const N: usize>(x: Simd<f64, N>) -> Simd<i32, N> {
    round_f64(x).cast()
}

/// Rounds each lane to the nearest `i64`, with ties away from zero.
///
/// Results outside the integer range (including infinities) saturate to
/// `i64::MIN` or `i64::MAX`; NaNs produce zero.
#[inline]
pub fn llround_f32<const N: usize>(x: Simd<f32, N>) -> Simd<i64, N> {
    round_f32(x).cast()
}

/// Rounds each lane to the nearest `i64`, with ties away from zero.
///
/// Results outside the integer range (including infinities) saturate to
/// `i64::MIN` or `i64::MAX`; NaNs produce zero.
#[inline]
pub fn llround_f64<const N: usize>(x: Simd<f64, N>) -> Simd<i64, N> {
    round_f64(x).cast()
}
