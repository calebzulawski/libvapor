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

pub(crate) fn normalize_f32<const N: usize>(x: Simd<f32, N>) -> Simd<u32, N> {
    let valid = x.is_finite() & x.simd_gt(Simd::splat(0.0));
    let a = valid.select(x, Simd::splat(1.0));
    let subnormal = a.is_subnormal();
    // Scale only subnormal lanes. Subtracting the scale's exponent
    // leaves a virtual exponent in wrapping integer arithmetic.
    let scaled = subnormal.select(a, Simd::splat(1.0));
    let scaled = scaled * Simd::splat(f32::from_bits((127u32 + 23u32) << 23));
    let bits = scaled.to_bits() - Simd::splat(23u32 << 23);
    subnormal.select(bits, a.to_bits())
}

pub(crate) fn finish_f32<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>) -> Simd<f32, N> {
    x.simd_eq(Simd::splat(0.0)).select(
        Simd::splat(f32::NEG_INFINITY),
        x.simd_lt(Simd::splat(0.0)).select(
            Simd::splat(f32::NAN),
            x.is_nan().select(
                x + x,
                x.is_infinite()
                    .select(x, x.simd_eq(Simd::splat(1.0)).select(Simd::splat(0.0), y)),
            ),
        ),
    )
}

pub(crate) fn interval_f32<const N: usize>(bits: Simd<u32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
    // x = 2^k * (1+f), with sqrt(2)/2 <= 1+f < sqrt(2).
    let bits = bits + Simd::splat((127u32 << 23) - 0x3f3504f3u32);
    let exponent = bits.cast::<i32>();
    let k = (exponent >> 23) - Simd::splat(127u32 as i32);
    let fraction_bits = bits & Simd::splat(((1 as u32) << 23) - 1);
    let reduced = Simd::<f32, N>::from_bits(fraction_bits + Simd::splat(0x3f3504f3u32));
    (k.cast::<i32>().cast::<f32>(), reduced - Simd::splat(1.0))
}
