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

// log(1+f) = f - hfsq + s*(hfsq+R). The callers retain these
// components separately for compensated reconstruction in their own bases.
pub(crate) fn series_f32<const N: usize>(
    f: Simd<f32, N>,
) -> (Simd<f32, N>, Simd<f32, N>, Simd<f32, N>) {
    let s = f / (Simd::splat(2.0) + f);
    let z = s * s;
    let w = z * z;
    let t1 = w * (Simd::splat(SERIES_F32[1]) + w * Simd::splat(SERIES_F32[3]));
    let t2 = z * (Simd::splat(SERIES_F32[0]) + w * Simd::splat(SERIES_F32[2]));
    (s, Simd::splat(0.5) * f * f, t2 + t1)
}
