/*
 * Derived from musl src/math/__sin.c and __cos.c, with changes.
 * SPDX-License-Identifier: MIT AND SunPro
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

use super::super::data::*;
use super::super::reduction::reduce_f64;
use super::finish::finish;
use core::simd::prelude::*;

fn kernels_f64<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    small: Mask<i64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    let z = x * x;
    let w = z * z;
    let r = Simd::splat(SIN_POLY_F64[1])
        + z * (Simd::splat(SIN_POLY_F64[2]) + z * Simd::splat(SIN_POLY_F64[3]))
        + z * w * (Simd::splat(SIN_POLY_F64[4]) + z * Simd::splat(SIN_POLY_F64[5]));
    let v = z * x;
    let sin = small.select(
        x + v * (Simd::splat(SIN_POLY_F64[0]) + z * r),
        x - ((z * (Simd::splat(0.5) * y - v * r) - y) - v * Simd::splat(SIN_POLY_F64[0])),
    );

    let r = z
        * (Simd::splat(COS_POLY_F64[0])
            + z * (Simd::splat(COS_POLY_F64[1]) + z * Simd::splat(COS_POLY_F64[2])))
        + w * w
            * (Simd::splat(COS_POLY_F64[3])
                + z * (Simd::splat(COS_POLY_F64[4]) + z * Simd::splat(COS_POLY_F64[5])));
    let hz = Simd::splat(0.5) * z;
    let w = Simd::splat(1.0) - hz;
    let cos = w + (((Simd::splat(1.0) - w) - hz) + (z * r - x * y));
    (sin, cos)
}

/// Computes (sin(x), cos(x)) in radians for each lane with shared argument reduction,
/// assuming round-to-nearest, ties-to-even.
#[inline]
pub(crate) fn full_range_sincos_f64<const N: usize>(
    x: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    let ax = x.is_finite().select(x.abs(), Simd::splat(0.0));
    let reduced = reduce_f64(ax);
    let (s, c) = kernels_f64(reduced.hi, reduced.lo, ax.simd_le(Simd::splat(PIO4)));
    finish(x, reduced.quadrant, s, c)
}
