/*
 * Derived from musl src/math/log1p.c and log2.c, with changes.
 * Base conversion: the fdlibm log2.c implementation in musl 1.1.19.
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

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

use super::super::super::reduction::*;
use super::super::super::series::*;
use core::simd::prelude::*;

#[inline]
fn normalize_f64<const N: usize>(x: Simd<f64, N>) -> Simd<u64, N> {
    let bits = x.to_bits();
    if (bits - Simd::splat(0x0010000000000000u64))
        .simd_ge(Simd::splat(0x7fe0000000000000u64))
        .any()
    {
        super::super::super::reduction::normalize_f64(x)
    } else {
        bits
    }
}

#[inline]
fn finish_f64<const N: usize>(x: Simd<f64, N>, y: Simd<f64, N>) -> Simd<f64, N> {
    let bits = x.to_bits();
    if (bits - Simd::splat(0x0010000000000000u64))
        .simd_ge(Simd::splat(0x7fe0000000000000u64))
        .any()
    {
        super::super::super::reduction::finish_f64(x, y)
    } else {
        y
    }
}

/// Computes the binary logarithm of each lane.
#[inline]
pub fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let (k, f) = interval_f64(normalize_f64(x));
    let (s, h, r) = series_f64(f);
    let hi = f - h;
    let lo = f - hi - h + s * (h + r);
    let y = (hi + lo) * Simd::splat(f64::from_bits(0x3de705fc2eefa200))
        + lo * Simd::splat(f64::from_bits(0x3ff7154765200000))
        + hi * Simd::splat(f64::from_bits(0x3ff7154765200000))
        + k;
    finish_f64(x, y)
}
