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

use super::super::super::data::*;
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

/// Computes the natural logarithm of each lane.
#[inline]
pub fn log_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let (k, f) = interval_f32(normalize_f32(x));
    let (s, h, r) = series_f32(f);
    let lo = s * (h + r) - h;
    let y = k * Simd::splat(SERIES_LN2_HI_F32) + (f + (lo + k * Simd::splat(SERIES_LN2_LO_F32)));
    finish_f32(x, y)
}
