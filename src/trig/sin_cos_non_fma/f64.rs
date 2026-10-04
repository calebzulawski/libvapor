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
use simd_macros::vectorize;

#[allow(unused_braces)]
fn kernels_f64<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    small: Mask<i64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let z = x * x;
        let w = z * z;
        let r = scalar!(SIN_POLY_F64[1])
            + z * (scalar!(SIN_POLY_F64[2]) + z * scalar!(SIN_POLY_F64[3]))
            + z * w * (scalar!(SIN_POLY_F64[4]) + z * scalar!(SIN_POLY_F64[5]));
        let v = z * x;
        let sin = if small {
            x + v * (scalar!(SIN_POLY_F64[0]) + z * r)
        } else {
            x - ((z * (0.5 * y - v * r) - y) - v * scalar!(SIN_POLY_F64[0]))
        };

        let r = z
            * (scalar!(COS_POLY_F64[0])
                + z * (scalar!(COS_POLY_F64[1]) + z * scalar!(COS_POLY_F64[2])))
            + w * w
                * (scalar!(COS_POLY_F64[3])
                    + z * (scalar!(COS_POLY_F64[4]) + z * scalar!(COS_POLY_F64[5])));
        let hz = 0.5 * z;
        let w = 1.0 - hz;
        let cos = w + (((1.0 - w) - hz) + (z * r - x * y));
        (sin, cos)
    })
}

#[allow(unused_braces)]
/// Computes (sin(x), cos(x)) in radians for each lane with shared argument reduction,
/// assuming round-to-nearest, ties-to-even.
#[inline]
pub(crate) fn full_range_sincos_f64<const N: usize>(
    x: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let ax = if x.is_finite() { x.abs() } else { 0.0 };
        let reduced = reduce_f64(ax);
        let (s, c) = kernels_f64(reduced.hi, reduced.lo, ax <= scalar!(PIO4));
        finish(x, reduced.quadrant, s, c)
    })
}
