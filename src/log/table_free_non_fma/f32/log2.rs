/*
 * Derived from musl src/math/log1pf.c and log2f.c, with changes.
 * Base conversion: the fdlibm log2f.c implementation in musl 1.1.19.
 * Inverse-ln(2) split adapted locally.
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

use super::super::super::reduction::*;
use super::super::super::series::*;
use core::simd::prelude::*;

#[inline]
fn normalize_f32<const N: usize>(x: Simd<f32, N>) -> Simd<u32, N> {
    let bits = x.to_bits();
    if (bits - Simd::splat(0x00800000u32))
        .simd_ge(Simd::splat(0x7f000000u32))
        .any()
    {
        super::super::super::reduction::normalize_f32(x)
    } else {
        bits
    }
}

#[inline]
fn finish_f32<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>) -> Simd<f32, N> {
    let bits = x.to_bits();
    if (bits - Simd::splat(0x00800000u32))
        .simd_ge(Simd::splat(0x7f000000u32))
        .any()
    {
        super::super::super::reduction::finish_f32(x, y)
    } else {
        y
    }
}

/// Computes the binary logarithm of each lane.
#[inline]
pub fn log2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let (k, f) = interval_f32(normalize_f32(x));
    let (s, h, r) = series_f32(f);
    let hi = f - h;
    let lo = f - hi - h + s * (h + r);
    let y = (hi + lo) * Simd::splat(f32::from_bits(0x36eca570))
        + lo * Simd::splat(f32::from_bits(0x3fb8aa00))
        + hi * Simd::splat(f32::from_bits(0x3fb8aa00))
        + k;
    finish_f32(x, y)
}
