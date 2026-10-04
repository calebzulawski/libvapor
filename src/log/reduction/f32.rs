/*
 * Derived from musl src/math/log1pf.c, with changes.
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
pub(crate) fn normalize_f32<const N: usize>(x: Simd<f32, N>) -> Simd<u32, N> {
    vectorize!(N, {
        let valid = x.is_finite() & (x > 0.0);
        let a = if valid { x } else { 1.0 };
        let subnormal = a.is_subnormal();
        // Scale only subnormal lanes. Subtracting the scale's exponent
        // leaves a virtual exponent in wrapping integer arithmetic.
        let scaled = if subnormal { a } else { 1.0 };
        let scaled = scaled * scalar!(f32::from_bits((127u32 + 23u32) << 23));
        let bits = scaled.to_bits() - scalar!(23u32 << 23);
        if subnormal {
            bits
        } else {
            a.to_bits()
        }
    })
}

#[allow(unused_braces)]
pub(crate) fn finish_f32<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        if x == 0.0 {
            scalar!(f32::NEG_INFINITY)
        } else if x < 0.0 {
            scalar!(f32::NAN)
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

pub(crate) fn interval_f32<const N: usize>(bits: Simd<u32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
    vectorize!(N, {
        // x = 2^k * (1+f), with sqrt(2)/2 <= 1+f < sqrt(2).
        let bits = bits + scalar!((127u32 << 23) - 0x3f3504f3u32);
        let exponent = bits as i32;
        let k = (exponent >> scalar!(23)) - scalar!(127u32 as i32);
        let fraction_bits = bits & scalar!(((1 as u32) << 23) - 1);
        let reduced = <f32>::from_bits(fraction_bits + scalar!(0x3f3504f3u32));
        (k as i32 as f32, reduced - 1.0)
    })
}
