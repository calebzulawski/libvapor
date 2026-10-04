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
use simd_macros::vectorize;

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
