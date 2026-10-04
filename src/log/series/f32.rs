/*
 * Derived from musl src/math/log1pf.c, with changes.
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

use super::super::data::SERIES_F32;
use core::simd::prelude::*;
use simd_macros::vectorize;

// log(1+f) = f - hfsq + s*(hfsq+R). The callers retain these
// components separately for compensated reconstruction in their own bases.
pub(crate) fn series_f32<const N: usize>(
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
