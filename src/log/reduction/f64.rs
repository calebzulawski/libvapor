/*
 * Derived from musl src/math/log1p.c, with changes.
 * SPDX-License-Identifier: MIT AND SunPro
 */

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

#[allow(unused_braces)]
pub(crate) fn normalize_f64<const N: usize>(x: Simd<f64, N>) -> Simd<u64, N> {
    vectorize!(N, {
        let valid = x.is_finite() & (x > 0.0);
        let a = if valid { x } else { 1.0 };
        let subnormal = a.is_subnormal();
        // Scale only subnormal lanes. Subtracting the scale's exponent
        // leaves a virtual exponent in wrapping integer arithmetic.
        let scaled = if subnormal { a } else { 1.0 };
        let scaled = scaled * scalar!(f64::from_bits((1023u64 + 52u64) << 52));
        let bits = scaled.to_bits() - scalar!(52u64 << 52);
        if subnormal {
            bits
        } else {
            a.to_bits()
        }
    })
}

#[allow(unused_braces)]
pub(crate) fn finish_f64<const N: usize>(x: Simd<f64, N>, y: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        if x == 0.0 {
            scalar!(f64::NEG_INFINITY)
        } else if x < 0.0 {
            scalar!(f64::NAN)
        } else if x.is_nan() {
            x + x
        } else if x.is_infinite() {
            x
        } else if x == 1.0 {
            0.0
        } else {
            y
        }
    })
}

pub(crate) fn interval_f64<const N: usize>(bits: Simd<u64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        // x = 2^k * (1+f), with sqrt(2)/2 <= 1+f < sqrt(2).
        let bits = bits + scalar!((1023u64 << 52) - 0x3fe6a09e00000000u64);
        let exponent = bits as i64;
        let k = (exponent >> scalar!(52)) - scalar!(1023u64 as i64);
        let fraction_bits = bits & scalar!(((1 as u64) << 52) - 1);
        let reduced = <f64>::from_bits(fraction_bits + scalar!(0x3fe6a09e00000000u64));
        (k as i32 as f64, reduced - 1.0)
    })
}
