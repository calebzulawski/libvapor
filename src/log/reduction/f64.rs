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

pub(crate) fn normalize_f64<const N: usize>(x: Simd<f64, N>) -> Simd<u64, N> {
    let valid = x.is_finite() & x.simd_gt(Simd::splat(0.0));
    let a = valid.select(x, Simd::splat(1.0));
    let subnormal = a.is_subnormal();
    // Scale only subnormal lanes. Subtracting the scale's exponent
    // leaves a virtual exponent in wrapping integer arithmetic.
    let scaled = subnormal.select(a, Simd::splat(1.0));
    let scaled = scaled * Simd::splat(f64::from_bits((1023u64 + 52u64) << 52));
    let bits = scaled.to_bits() - Simd::splat(52u64 << 52);
    subnormal.select(bits, a.to_bits())
}

pub(crate) fn finish_f64<const N: usize>(x: Simd<f64, N>, y: Simd<f64, N>) -> Simd<f64, N> {
    x.simd_eq(Simd::splat(0.0)).select(
        Simd::splat(f64::NEG_INFINITY),
        x.simd_lt(Simd::splat(0.0)).select(
            Simd::splat(f64::NAN),
            x.is_nan().select(
                x + x,
                x.is_infinite()
                    .select(x, x.simd_eq(Simd::splat(1.0)).select(Simd::splat(0.0), y)),
            ),
        ),
    )
}

pub(crate) fn interval_f64<const N: usize>(bits: Simd<u64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    // x = 2^k * (1+f), with sqrt(2)/2 <= 1+f < sqrt(2).
    let bits = bits + Simd::splat((1023u64 << 52) - 0x3fe6a09e00000000u64);
    let exponent = bits.cast::<i64>();
    let k = (exponent >> 52) - Simd::splat(1023u64 as i64);
    let fraction_bits = bits & Simd::splat(((1 as u64) << 52) - 1);
    let reduced = Simd::<f64, N>::from_bits(fraction_bits + Simd::splat(0x3fe6a09e00000000u64));
    (k.cast::<i32>().cast::<f64>(), reduced - Simd::splat(1.0))
}
