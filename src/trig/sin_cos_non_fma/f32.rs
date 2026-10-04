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
use simd_macros::vectorize;

fn kernels_f32<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let z = x * x;
        let w = z * z;
        let r = scalar!(SIN_POLY_F32[2]) + z * scalar!(SIN_POLY_F32[3]);
        let s = z * x;
        let sin = (x + s * (scalar!(SIN_POLY_F32[0]) + z * scalar!(SIN_POLY_F32[1]))) + s * w * r;
        let r = scalar!(COS_POLY_F32[2]) + z * scalar!(COS_POLY_F32[3]);
        let cos =
            ((1.0 + z * scalar!(COS_POLY_F32[0])) + w * scalar!(COS_POLY_F32[1])) + (w * z) * r;
        (sin, cos)
    })
}

#[allow(unused_braces)]
/// Computes (sin(x), cos(x)) in radians for each lane with shared argument reduction,
/// assuming round-to-nearest, ties-to-even.
#[inline]
pub(crate) fn full_range_sincos_f32<const N: usize>(
    x: Simd<f32, N>,
) -> (Simd<f32, N>, Simd<f32, N>) {
    vectorize!(N, {
        let xd = x as f64;
        let ax = if xd.is_finite() { xd.abs() } else { 0.0 };
        let bits = x.to_bits() as u64;
        let reduced = reduce_f32(ax, bits);
        let (s, c) = kernels_f32(reduced.hi);
        let (sin, cos) = finish(xd, reduced.quadrant, s, c);
        (sin as f32, cos as f32)
    })
}
