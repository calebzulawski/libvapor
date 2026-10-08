/*
 * Derived from musl src/math/__tandf.c, with changes.
 * SPDX-License-Identifier: MIT AND LicenseRef-SunPro-short
 */

/*
 * ====================================================
 * Copyright 2004 Sun Microsystems, Inc.  All Rights Reserved.
 *
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

/*
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
 * Debugged and optimized by Bruce D. Evans.
 */

use super::super::data::*;
use super::super::reduction::reduce_f32;
use core::simd::prelude::*;

fn tan_kernel_f32<const N: usize>(x: Simd<f64, N>, odd: Mask<i64, N>) -> Simd<f64, N> {
    let z = x * x;
    let r = Simd::splat(TAN_POLY_F32[4]) + z * Simd::splat(TAN_POLY_F32[5]);
    let t = Simd::splat(TAN_POLY_F32[2]) + z * Simd::splat(TAN_POLY_F32[3]);
    let w = z * z;
    let s = z * x;
    let u = Simd::splat(TAN_POLY_F32[0]) + z * Simd::splat(TAN_POLY_F32[1]);
    let r = (x + s * u) + (s * w) * (t + w * r);
    // Odd quadrants use -cot(x); round to f32 only after division.
    let denominator = odd.select(r, Simd::splat(1.0));
    odd.select(Simd::splat(-1.0) / denominator, r)
}

/// Computes tan(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn tan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let xd = x.cast::<f64>();
    let finite = xd.is_finite();
    let ax = finite.select(xd.abs(), Simd::splat(0.0));
    let bits = x.to_bits().cast::<u64>();
    let reduced = reduce_f32(ax, bits);
    let odd = (reduced.quadrant & Simd::splat(1)).simd_ne(Simd::splat(0));
    let tan = tan_kernel_f32(reduced.hi, odd);
    let tan = Simd::<f64, N>::from_bits(tan.to_bits() ^ (xd.to_bits() & (Simd::splat(1u64) << 63)));
    let tan = finite.select(tan, xd - xd);
    tan.cast::<f32>()
}
