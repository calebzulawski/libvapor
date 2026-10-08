/*
 * Derived from musl src/math/__tan.c, with changes.
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

use super::super::data::*;
use super::super::reduction::reduce_f64;
use core::simd::prelude::*;

fn tan_kernel_f64<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    odd: Mask<i64, N>,
) -> Simd<f64, N> {
    let (x, r, w, big, sign) = {
        let big = x.abs().simd_ge(Simd::splat(TAN_TRANSFORM_LIMIT));
        let sign = x.is_sign_negative();
        let negate = big & sign;
        let x = negate.select(-x, x);
        let y = negate.select(-y, y);
        // Near pi/4, evaluate tan(pi/4 - x) on a smaller interval.
        let x = big.select((Simd::splat(PIO4) - x) + (Simd::splat(PIO4_LO) - y), x);
        let y = big.select(Simd::splat(0.0), y);
        let z = x * x;
        let w = z * z;
        let r = Simd::splat(TAN_POLY_F64[1])
            + w * (Simd::splat(TAN_POLY_F64[3])
                + w * (Simd::splat(TAN_POLY_F64[5])
                    + w * (Simd::splat(TAN_POLY_F64[7])
                        + w * (Simd::splat(TAN_POLY_F64[9]) + w * Simd::splat(TAN_POLY_F64[11])))));
        let v = z
            * (Simd::splat(TAN_POLY_F64[2])
                + w * (Simd::splat(TAN_POLY_F64[4])
                    + w * (Simd::splat(TAN_POLY_F64[6])
                        + w * (Simd::splat(TAN_POLY_F64[8])
                            + w * (Simd::splat(TAN_POLY_F64[10])
                                + w * Simd::splat(TAN_POLY_F64[12]))))));
        let s = z * x;
        let r = y + z * (s * (r + v) + y) + s * Simd::splat(TAN_POLY_F64[0]);
        let w = x + r;
        (x, r, w, big, sign)
    };
    let result = if big.any() {
        let s = odd.select(Simd::splat(-1.0), Simd::splat(1.0));
        let transformed = s - Simd::splat(2.0) * (x + (r - w * w / (w + s)));
        let transformed = sign.select(-transformed, transformed);
        big.select(transformed, w)
    } else {
        w
    };
    let reciprocal = odd & !big;
    if !reciprocal.any() {
        return result;
    }

    // Correct the reciprocal using the polynomial's rounding residual;
    // a plain -1/w can lose two ULPs. Keep inactive denominators nonzero.
    let denominator = reciprocal.select(w, Simd::splat(1.0));
    let w0 = Simd::<f64, N>::from_bits(denominator.to_bits() & Simd::splat(0xffffffff00000000));
    let v = r - (w0 - x);
    let a: Simd<f64, N> = Simd::splat(-1.0) / denominator;
    let a0 = Simd::<f64, N>::from_bits(a.to_bits() & Simd::splat(0xffffffff00000000));
    let corrected = a0 + a * (Simd::splat(1.0) + a0 * w0 + a0 * v);
    reciprocal.select(corrected, result)
}

/// Computes tan(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn tan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let finite = x.is_finite();
    let ax = finite.select(x.abs(), Simd::splat(0.0));
    let reduced = reduce_f64(ax);
    let odd = (reduced.quadrant & Simd::splat(1)).simd_ne(Simd::splat(0));
    let tan = tan_kernel_f64(reduced.hi, reduced.lo, odd);
    let tan = Simd::<f64, N>::from_bits(tan.to_bits() ^ (x.to_bits() & (Simd::splat(1u64) << 63)));
    finite.select(tan, x - x)
}
