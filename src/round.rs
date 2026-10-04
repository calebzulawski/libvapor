//! Portable SIMD rounding and fractional parts.
mod hardware;
mod software;

use core::simd::Simd;

/// Computes rounding toward zero for each lane.
#[inline]
pub fn trunc_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::trunc_f32(x)
    } else {
        software::trunc_f32(x)
    }
}

/// Computes rounding toward zero for each lane.
#[inline]
pub fn trunc_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::trunc_f64(x)
    } else {
        software::trunc_f64(x)
    }
}

/// Computes the fractional part of each lane.
#[inline]
pub fn fract_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::fract_f32(x)
    } else {
        software::fract_f32(x)
    }
}
/// Computes the fractional part of each lane.
#[inline]
pub fn fract_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::fract_f64(x)
    } else {
        software::fract_f64(x)
    }
}

/// Computes rounding toward negative infinity for each lane.
#[inline]
pub fn floor_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::floor_f32(x)
    } else {
        software::floor_f32(x)
    }
}

/// Computes rounding toward negative infinity for each lane.
#[inline]
pub fn floor_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::floor_f64(x)
    } else {
        software::floor_f64(x)
    }
}

/// Computes rounding toward positive infinity for each lane.
#[inline]
pub fn ceil_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::ceil_f32(x)
    } else {
        software::ceil_f32(x)
    }
}

/// Computes rounding toward positive infinity for each lane.
#[inline]
pub fn ceil_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::ceil_f64(x)
    } else {
        software::ceil_f64(x)
    }
}

/// Computes rounding to nearest with ties away from zero for each lane.
#[inline]
pub fn round_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::round_f32(x)
    } else {
        software::round_f32(x)
    }
}

/// Computes rounding to nearest with ties away from zero for each lane.
#[inline]
pub fn round_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_ROUND {
        hardware::round_f64(x)
    } else {
        software::round_f64(x)
    }
}
