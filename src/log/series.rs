/*
 * Adapted from musl libc: log1p.c, log1pf.c, log10.c, and log10f.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::{SERIES_F32, SERIES_F64};

// log(1+f) = f - hfsq + s*(hfsq+R). The callers retain these
// components separately for compensated reconstruction in their own bases.
pub(super) fn series_f32<const N: usize>(
    f: Simd<f32, N>,
) -> (Simd<f32, N>, Simd<f32, N>, Simd<f32, N>) {
    vectorize!(N, {
        let s = f / (2.0 + f);
        let z = s * s;
        let w = z * z;
        let t1 = w * (scalar!(SERIES_F32[1]) + w * scalar!(SERIES_F32[3]));
        let t2 = z * (scalar!(SERIES_F32[0]) + w * scalar!(SERIES_F32[2]));
        (s, 0.5 * f * f, t2 + t1)
    })
}

pub(super) fn series_f64<const N: usize>(
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
