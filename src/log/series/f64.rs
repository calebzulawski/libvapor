/*
 * Derived from musl src/math/log1p.c, with changes.
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

use super::super::data::SERIES_F64;
use core::simd::prelude::*;

pub(crate) fn series_f64<const N: usize>(
    f: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>, Simd<f64, N>) {
    let s = f / (Simd::splat(2.0) + f);
    let z = s * s;
    let w = z * z;
    let t1 = w
        * (Simd::splat(SERIES_F64[1])
            + w * (Simd::splat(SERIES_F64[3]) + w * Simd::splat(SERIES_F64[5])));
    let t2 = z
        * (Simd::splat(SERIES_F64[0])
            + w * (Simd::splat(SERIES_F64[2])
                + w * (Simd::splat(SERIES_F64[4]) + w * Simd::splat(SERIES_F64[6]))));
    (s, Simd::splat(0.5) * f * f, t2 + t1)
}
