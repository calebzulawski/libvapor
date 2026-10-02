/*
 * Kernels and medium-range reduction adapted from musl libc.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Copyright 2004 Sun Microsystems, Inc. All Rights Reserved.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 *
 * Large f32 reduction adapted from Arm optimized-routines: sincosf.h.
 * Copyright (c) 2018-2024, Arm Limited.
 * SPDX-License-Identifier: MIT
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::*;
use super::reduction::{reduce_f32, reduce_f64};

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
fn finish<const N: usize>(
    x: Simd<f64, N>,
    quadrant: Simd<u64, N>,
    s: Simd<f64, N>,
    c: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let sin = if quadrant & 1 == 0 { s } else { c };
        let sin = if quadrant & 2 == 0 { sin } else { -sin };
        let sin = <f64>::from_bits(sin.to_bits() ^ (x.to_bits() & (1u64 << 63)));
        let cos = if quadrant & 1 == 0 { c } else { s };
        let cos_quadrant = quadrant + 1;
        let cos = if cos_quadrant & 2 == 0 { cos } else { -cos };
        let finite = x.is_finite();
        let sin = if finite { sin } else { x - x };
        let cos = if finite { cos } else { x - x };
        (sin, cos)
    })
}

#[allow(unused_braces)]
/// Computes (sin(x), cos(x)) in radians for each lane with shared argument reduction,
/// assuming round-to-nearest, ties-to-even.
#[inline]
pub fn sincos_f32<const N: usize>(x: Simd<f32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
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

#[allow(unused_braces)]
/// Computes (sin(x), cos(x)) in radians for each lane with shared argument reduction,
/// assuming round-to-nearest, ties-to-even.
#[inline]
pub fn sincos_f64<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let ax = if x.is_finite() { x.abs() } else { 0.0 };
        let reduced = reduce_f64(ax);
        let (s, c) = kernels_f64(reduced.hi, reduced.lo, ax <= scalar!(PIO4));
        finish(x, reduced.quadrant, s, c)
    })
}

/// Computes sin(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn sin_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    sincos_f32(x).0
}

/// Computes cos(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn cos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    sincos_f32(x).1
}

/// Computes sin(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn sin_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    sincos_f64(x).0
}

/// Computes cos(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn cos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    sincos_f64(x).1
}
