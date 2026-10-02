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

#[allow(unused_braces)]
fn tan_kernel_f32<const N: usize>(x: Simd<f64, N>, odd: Mask<i64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let z = x * x;
        let r = scalar!(TAN_POLY_F32[4]) + z * scalar!(TAN_POLY_F32[5]);
        let t = scalar!(TAN_POLY_F32[2]) + z * scalar!(TAN_POLY_F32[3]);
        let w = z * z;
        let s = z * x;
        let u = scalar!(TAN_POLY_F32[0]) + z * scalar!(TAN_POLY_F32[1]);
        let r = (x + s * u) + (s * w) * (t + w * r);
        // Odd quadrants use -cot(x); round to f32 only after division.
        let denominator = if odd { r } else { 1.0 };
        if odd {
            -1.0 / denominator
        } else {
            r
        }
    })
}

#[allow(unused_braces)]
fn tan_kernel_f64<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    odd: Mask<i64, N>,
) -> Simd<f64, N> {
    let (x, r, w, big, sign) = vectorize!(N, {
        let big = x.abs() >= scalar!(TAN_TRANSFORM_LIMIT);
        let sign = x.is_sign_negative();
        let negate = big & sign;
        let x = if negate { -x } else { x };
        let y = if negate { -y } else { y };
        // Near pi/4, evaluate tan(pi/4 - x) on a smaller interval.
        let x = if big {
            (scalar!(PIO4) - x) + (scalar!(PIO4_LO) - y)
        } else {
            x
        };
        let y = if big { 0.0 } else { y };
        let z = x * x;
        let w = z * z;
        let r = scalar!(TAN_POLY_F64[1])
            + w * (scalar!(TAN_POLY_F64[3])
                + w * (scalar!(TAN_POLY_F64[5])
                    + w * (scalar!(TAN_POLY_F64[7])
                        + w * (scalar!(TAN_POLY_F64[9]) + w * scalar!(TAN_POLY_F64[11])))));
        let v = z
            * (scalar!(TAN_POLY_F64[2])
                + w * (scalar!(TAN_POLY_F64[4])
                    + w * (scalar!(TAN_POLY_F64[6])
                        + w * (scalar!(TAN_POLY_F64[8])
                            + w * (scalar!(TAN_POLY_F64[10]) + w * scalar!(TAN_POLY_F64[12]))))));
        let s = z * x;
        let r = y + z * (s * (r + v) + y) + s * scalar!(TAN_POLY_F64[0]);
        let w = x + r;
        (x, r, w, big, sign)
    });
    let result = if big.any() {
        vectorize!(N, {
            let s = if odd { -1.0 } else { 1.0 };
            let transformed = s - 2.0 * (x + (r - w * w / (w + s)));
            let transformed = if sign { -transformed } else { transformed };
            if big {
                transformed
            } else {
                w
            }
        })
    } else {
        w
    };
    let reciprocal = odd & !big;
    if !reciprocal.any() {
        return result;
    }
    vectorize!(N, {
        // Correct the reciprocal using the polynomial's rounding residual;
        // a plain -1/w can lose two ULPs. Keep inactive denominators nonzero.
        let denominator = if reciprocal { w } else { 1.0 };
        let w0 = <f64>::from_bits(denominator.to_bits() & 0xffffffff00000000);
        let v = r - (w0 - x);
        let a: f64 = -1.0 / denominator;
        let a0 = <f64>::from_bits(a.to_bits() & 0xffffffff00000000);
        let corrected = a0 + a * (1.0 + a0 * w0 + a0 * v);
        if reciprocal {
            corrected
        } else {
            result
        }
    })
}

#[allow(unused_braces)]
/// Computes tan(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn tan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let xd = x as f64;
        let finite = xd.is_finite();
        let ax = if finite { xd.abs() } else { 0.0 };
        let bits = x.to_bits() as u64;
        let reduced = reduce_f32(ax, bits);
        let odd = reduced.quadrant & 1 != 0;
        let tan = tan_kernel_f32(reduced.hi, odd);
        let tan = <f64>::from_bits(tan.to_bits() ^ (xd.to_bits() & (1u64 << 63)));
        let tan = if finite { tan } else { xd - xd };
        tan as f32
    })
}

#[allow(unused_braces)]
/// Computes tan(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn tan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let finite = x.is_finite();
        let ax = if finite { x.abs() } else { 0.0 };
        let reduced = reduce_f64(ax);
        let odd = reduced.quadrant & 1 != 0;
        let tan = tan_kernel_f64(reduced.hi, reduced.lo, odd);
        let tan = <f64>::from_bits(tan.to_bits() ^ (x.to_bits() & (1u64 << 63)));
        if finite {
            tan
        } else {
            x - x
        }
    })
}
