//! Portable SIMD trigonometric functions and their shared kernels.
pub(crate) mod data;
pub(crate) mod inverse_fma;
pub(crate) mod inverse_non_fma;
pub(crate) mod reduction;
pub(crate) mod sin_cos_fma;
pub(crate) mod sin_cos_non_fma;
pub(crate) mod sin_cos_non_fma_fast;
pub(crate) mod sin_cos_pair_fma;
pub(crate) mod sin_cos_pair_non_fma;
pub(crate) mod tan;
pub(crate) mod tan_fma;

use core::simd::Simd;

/// Computes sin(x) for each lane.
#[inline]
pub fn sin_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        sin_cos_fma::sin_cos_f32::<N, false>(x)
    } else {
        sin_cos_non_fma_fast::sin_cos_f32::<N, false>(x)
    }
}

/// Computes sin(x) for each lane.
#[inline]
pub fn sin_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        sin_cos_fma::sin_cos_f64::<N, false>(x)
    } else {
        sin_cos_non_fma::full_range_sincos_f64(x).0
    }
}

/// Computes cos(x) for each lane.
#[inline]
pub fn cos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        sin_cos_fma::sin_cos_f32::<N, true>(x)
    } else {
        sin_cos_non_fma_fast::sin_cos_f32::<N, true>(x)
    }
}

/// Computes cos(x) for each lane.
#[inline]
pub fn cos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        sin_cos_fma::sin_cos_f64::<N, true>(x)
    } else {
        sin_cos_non_fma::full_range_sincos_f64(x).1
    }
}

/// Computes (sin(x), cos(x)) for each lane.
#[inline]
pub fn sincos_f32<const N: usize>(x: Simd<f32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
    if crate::backend::USE_HARDWARE_FMA {
        sin_cos_pair_fma::sincos_f32(x)
    } else {
        sin_cos_pair_non_fma::sincos_f32(x)
    }
}

/// Computes (sin(x), cos(x)) for each lane.
#[inline]
pub fn sincos_f64<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    if crate::backend::USE_HARDWARE_FMA {
        sin_cos_pair_fma::sincos_f64(x)
    } else {
        sin_cos_non_fma::full_range_sincos_f64(x)
    }
}

/// Computes tan(x) for each lane.
#[inline]
pub fn tan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        tan_fma::tan_f32(x)
    } else {
        tan::tan_f32(x)
    }
}

/// Computes tan(x) for each lane.
#[inline]
pub fn tan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        tan_fma::tan_f64(x)
    } else {
        tan::tan_f64(x)
    }
}

/// Computes asin(x) for each lane.
#[inline]
pub fn asin_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::asin_f32(x)
    } else {
        inverse_non_fma::asin_f32(x)
    }
}

/// Computes asin(x) for each lane.
#[inline]
pub fn asin_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::asin_f64(x)
    } else {
        inverse_non_fma::asin_f64(x)
    }
}

/// Computes acos(x) for each lane.
#[inline]
pub fn acos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::acos_f32(x)
    } else {
        inverse_non_fma::acos_f32(x)
    }
}

/// Computes acos(x) for each lane.
#[inline]
pub fn acos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::acos_f64(x)
    } else {
        inverse_non_fma::acos_f64(x)
    }
}

/// Computes atan(x) for each lane.
#[inline]
pub fn atan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::atan_f32(x)
    } else {
        inverse_non_fma::atan_f32(x)
    }
}

/// Computes atan(x) for each lane.
#[inline]
pub fn atan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::atan_f64(x)
    } else {
        inverse_non_fma::atan_f64(x)
    }
}

/// Computes atan2(y,x) for each lane.
#[inline]
pub fn atan2_f32<const N: usize>(y: Simd<f32, N>, x: Simd<f32, N>) -> Simd<f32, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::atan2_f32(y, x)
    } else {
        inverse_non_fma::atan2_f32(y, x)
    }
}

/// Computes atan2(y,x) for each lane.
#[inline]
pub fn atan2_f64<const N: usize>(y: Simd<f64, N>, x: Simd<f64, N>) -> Simd<f64, N> {
    if crate::backend::USE_HARDWARE_FMA {
        inverse_fma::atan2_f64(y, x)
    } else {
        inverse_non_fma::atan2_f64(y, x)
    }
}
