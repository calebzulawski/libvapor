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
pub(crate) mod table_non_fma;

use core::simd::Simd;

// The split-center table beats the table-free method on baseline x86 and Wasm.
// Keep other targets on their existing implementations until benchmarked.
const USE_TABLE_NON_FMA_F64: bool = cfg!(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx")
    ),
    target_arch = "wasm32",
    target_arch = "wasm64"
));

/// Computes the natural logarithm for each lane.
#[inline]
pub fn log_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_fma_f32::<N>() {
        polynomial_fma::log_f32(x)
    } else {
        table_free_non_fma::log_f32(x)
    }
}

/// Computes the natural logarithm for each lane.
#[inline]
pub fn log_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_fma_f64::<N>() {
        table_fma::log_f64(x)
    } else if USE_TABLE_NON_FMA_F64 {
        table_non_fma::log_f64(x)
    } else {
        table_free_non_fma::log_f64(x)
    }
}

/// Computes the base-two logarithm for each lane.
#[inline]
pub fn log2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_fma_f32::<N>() {
        polynomial_fma::log2_f32(x)
    } else {
        table_free_non_fma::log2_f32(x)
    }
}

/// Computes the base-two logarithm for each lane.
#[inline]
pub fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_fma_f64::<N>() {
        table_fma::log2_f64(x)
    } else if USE_TABLE_NON_FMA_F64 {
        table_non_fma::log2_f64(x)
    } else {
        table_free_non_fma::log2_f64(x)
    }
}

/// Computes the base-ten logarithm for each lane.
#[inline]
pub fn log10_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_fma_f32::<N>() {
        polynomial_fma::log10_f32(x)
    } else {
        table_free_non_fma::log10_f32(x)
    }
}

/// Computes the base-ten logarithm for each lane.
#[inline]
pub fn log10_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_fma_f64::<N>() {
        table_fma::log10_f64(x)
    } else {
        table_free_non_fma::log10_f64(x)
    }
}

/// Computes ln(1+x) for each lane.
#[inline]
pub fn log1p_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::use_hardware_fma_f32::<N>() {
        one_plus_fma::log1p_f32(x)
    } else {
        one_plus_non_fma::log1p_f32(x)
    }
}

/// Computes ln(1+x) for each lane.
#[inline]
pub fn log1p_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::use_hardware_fma_f64::<N>() {
        one_plus_fma::log1p_f64(x)
    } else {
        one_plus_non_fma::log1p_f64(x)
    }
}
