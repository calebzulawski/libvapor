/*
 * Derived from musl src/math/__sindf.c and __cosdf.c, with changes.
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
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
 * Debugged and optimized by Bruce D. Evans.
 */

use super::super::data::*;
use super::super::reduction::reduce_f32;
use super::finish::finish;
use core::simd::prelude::*;

fn kernels_f32<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    let z = x * x;
    let w = z * z;
    let r = Simd::splat(SIN_POLY_F32[2]) + z * Simd::splat(SIN_POLY_F32[3]);
    let s = z * x;
    let sin =
        (x + s * (Simd::splat(SIN_POLY_F32[0]) + z * Simd::splat(SIN_POLY_F32[1]))) + s * w * r;
    let r = Simd::splat(COS_POLY_F32[2]) + z * Simd::splat(COS_POLY_F32[3]);
    let cos = ((Simd::splat(1.0) + z * Simd::splat(COS_POLY_F32[0]))
        + w * Simd::splat(COS_POLY_F32[1]))
        + (w * z) * r;
    (sin, cos)
}

/// Computes (sin(x), cos(x)) in radians for each lane with shared argument reduction,
/// assuming round-to-nearest, ties-to-even.
#[inline]
pub(crate) fn full_range_sincos_f32<const N: usize>(
    x: Simd<f32, N>,
) -> (Simd<f32, N>, Simd<f32, N>) {
    let xd = x.cast::<f64>();
    let ax = xd.is_finite().select(xd.abs(), Simd::splat(0.0));
    let bits = x.to_bits().cast::<u64>();
    let reduced = reduce_f32(ax, bits);
    let (s, c) = kernels_f32(reduced.hi);
    let (sin, cos) = finish(xd, reduced.quadrant, s, c);
    (sin.cast::<f32>(), cos.cast::<f32>())
}
