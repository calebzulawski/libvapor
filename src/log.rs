//! Portable SIMD logarithms.
//! Implementation selection is confined to these public entry points.
pub(crate) mod data;
pub(crate) mod one_plus_fma;
pub(crate) mod one_plus_non_fma;
pub(crate) mod polynomial_fma;
pub(crate) mod reduction;
pub(crate) mod series;
pub(crate) mod table_fma;
pub(crate) mod table_free_non_fma;

use core::simd::Simd;

/// Computes the natural logarithm for each lane.
#[inline]
pub fn log_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        polynomial_fma::log_f32(x)
    } else {
        table_free_non_fma::log_f32(x)
    }
}

/// Computes the natural logarithm for each lane.
#[inline]
pub fn log_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        table_fma::log_f64(x)
    } else {
        table_free_non_fma::log_f64(x)
    }
}

/// Computes the base-two logarithm for each lane.
#[inline]
pub fn log2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        polynomial_fma::log2_f32(x)
    } else {
        table_free_non_fma::log2_f32(x)
    }
}

/// Computes the base-two logarithm for each lane.
#[inline]
pub fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        table_fma::log2_f64(x)
    } else {
        table_free_non_fma::log2_f64(x)
    }
}

/// Computes the base-ten logarithm for each lane.
#[inline]
pub fn log10_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        polynomial_fma::log10_f32(x)
    } else {
        table_free_non_fma::log10_f32(x)
    }
}

/// Computes the base-ten logarithm for each lane.
#[inline]
pub fn log10_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        table_fma::log10_f64(x)
    } else {
        table_free_non_fma::log10_f64(x)
    }
}

/// Computes ln(1+x) for each lane.
#[inline]
pub fn log1p_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        one_plus_fma::log1p_f32(x)
    } else {
        one_plus_non_fma::log1p_f32(x)
    }
}

/// Computes ln(1+x) for each lane.
#[inline]
pub fn log1p_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        one_plus_fma::log1p_f64(x)
    } else {
        one_plus_non_fma::log1p_f64(x)
    }
}
