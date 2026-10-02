/*
 * Adapted from musl libc: atan.c and atanf.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::*;

fn correction_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let z = x * x;
        let w = z * z;
        let s1 = z
            * (scalar!(ATAN_POLY_F32[0])
                + w * (scalar!(ATAN_POLY_F32[2]) + w * scalar!(ATAN_POLY_F32[4])));
        let s2 = w * (scalar!(ATAN_POLY_F32[1]) + w * scalar!(ATAN_POLY_F32[3]));
        x * (s1 + s2)
    })
}

fn correction_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let z = x * x;
        let w = z * z;
        let s1 = z
            * (scalar!(ATAN_POLY_F64[0])
                + w * (scalar!(ATAN_POLY_F64[2])
                    + w * (scalar!(ATAN_POLY_F64[4])
                        + w * (scalar!(ATAN_POLY_F64[6])
                            + w * (scalar!(ATAN_POLY_F64[8]) + w * scalar!(ATAN_POLY_F64[10]))))));
        let s2 = w
            * (scalar!(ATAN_POLY_F64[1])
                + w * (scalar!(ATAN_POLY_F64[3])
                    + w * (scalar!(ATAN_POLY_F64[5])
                        + w * (scalar!(ATAN_POLY_F64[7]) + w * scalar!(ATAN_POLY_F64[9])))));
        x * (s1 + s2)
    })
}

macro_rules! make_helpers {
    ($name:ident, $float:ident, $uint:ident, $hi:ident, $lo:ident,
     $correction:ident, $large:expr, $tiny:expr) => {
        // vectorize! retains scalar if-branch braces in select arguments.
        #[allow(unused_braces)]
        /// Computes atan(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
        #[inline]
        pub fn $name<const N: usize>(x: Simd<$float, N>) -> Simd<$float, N> {
            vectorize!(N, {
                let ax = x.abs();
                let active = x.is_finite() & (ax < scalar!($large));
                let a = if active { ax } else { 0.0 };
                let small = a < 0.4375;
                // Select one rational transformation before dividing, rather
                // than evaluating a division for every reduction interval.
                let numerator = if small {
                    a
                } else if a < 0.6875 {
                    2.0 * a - 1.0
                } else if a < 1.1875 {
                    a - 1.0
                } else if a < 2.4375 {
                    a - 1.5
                } else {
                    -1.0
                };
                let denominator = if small {
                    1.0
                } else if a < 0.6875 {
                    2.0 + a
                } else if a < 1.1875 {
                    a + 1.0
                } else if a < 2.4375 {
                    1.0 + 1.5 * a
                } else {
                    a
                };
                let t = numerator / denominator;
                let id: $uint = if a < 0.6875 {
                    0
                } else if a < 1.1875 {
                    1
                } else if a < 2.4375 {
                    2
                } else {
                    3
                };
                let hi = <$float>::gather_or(&$hi, id as usize, 0.0);
                let lo = <$float>::gather_or(&$lo, id as usize, 0.0);
                let correction = $correction(t);
                let result = if small {
                    t - correction
                } else {
                    hi - ((correction - lo) - t)
                };
                let result = if active {
                    result
                } else {
                    scalar!($hi[3]) + scalar!($lo[3])
                };
                let result = if ax < scalar!($tiny) { ax } else { result };
                let sign = x.to_bits() & scalar!((1 as $uint) << ($uint::BITS - 1));
                let result = <$float>::from_bits(result.to_bits() ^ sign);
                if x.is_nan() {
                    x + x
                } else {
                    result
                }
            })
        }
    };
}

make_helpers!(
    atan_f32,
    f32,
    u32,
    ATAN_HI_F32,
    ATAN_LO_F32,
    correction_f32,
    f32::from_bits(0x4c800000),
    f32::from_bits(0x39800000)
);
make_helpers!(
    atan_f64,
    f64,
    u64,
    ATAN_HI_F64,
    ATAN_LO_F64,
    correction_f64,
    f64::from_bits(0x4410000000000000),
    f64::from_bits(0x3e40000000000000)
);
