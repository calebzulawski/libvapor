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
use simd_macros::vectorize;

pub(crate) fn series_f64<const N: usize>(
    f: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let s = f / (2.0 + f);
        let z = s * s;
        let w = z * z;
        let t1 = w
            * (scalar!(SERIES_F64[1]) + w * (scalar!(SERIES_F64[3]) + w * scalar!(SERIES_F64[5])));
        let t2 = z
            * (scalar!(SERIES_F64[0])
                + w * (scalar!(SERIES_F64[2])
                    + w * (scalar!(SERIES_F64[4]) + w * scalar!(SERIES_F64[6]))));
        (s, 0.5 * f * f, t2 + t1)
    })
}
