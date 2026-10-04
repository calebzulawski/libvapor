/*
 * Derived from musl src/math/log1p.c and log10.c, with changes.
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

use super::super::super::data::*;
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

/// Computes the decimal logarithm of each lane.
#[inline]
pub fn log10_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let (k, f) = interval_f64(normalize_f64(x));
    let (s, h, r) = series_f64(f);
    let hi = f - h;
    let lo = f - hi - h + s * (h + r);
    let y = k * Simd::splat(LOG10_2_HI_F64) + hi * Simd::splat(INV_LN10_HI_F64);
    let lo = k * Simd::splat(LOG10_2_LO_F64)
        + (hi + lo) * Simd::splat(INV_LN10_LO_F64)
        + lo * Simd::splat(INV_LN10_HI_F64);
    finish_f64(x, y + lo)
}
