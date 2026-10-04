/*
 * Derived from musl src/math/__rem_pio2f.c, with changes.
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

/*
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
 * Debugged and optimized by Bruce D. Evans.
 */

use super::super::data::*;
use super::large_f32::reduce_large_f32;
use super::{round_quadrant, Reduced};
use core::simd::prelude::*;
use simd_macros::vectorize;

#[allow(unused_braces)]
pub(crate) fn reduce_f32<const N: usize>(ax: Simd<f64, N>, bits: Simd<u64, N>) -> Reduced<N> {
    let large = ax.simd_ge(Simd::splat(REDUCTION_LIMIT_F32));
    let (quadrant, hi) = vectorize!(N, {
        let ax = if large { 0.0 } else { ax };
        let (quadrant, kd) = round_quadrant(ax);
        let hi = ax - kd * scalar!(PIO2_REDUCE_HI_F32) - kd * scalar!(PIO2_REDUCE_LO_F32);
        (quadrant, hi)
    });
    if !large.any() {
        return Reduced {
            quadrant,
            hi,
            lo: Simd::splat(0.0),
        };
    }
    // Keep inactive lanes within the large reducer's domain (x >= 2).
    let large_bits = large.select(bits, Simd::splat(0x40000000));
    let reduced = reduce_large_f32(large_bits);
    Reduced {
        quadrant: large.select(reduced.quadrant, quadrant),
        hi: large.select(reduced.hi, hi),
        lo: Simd::splat(0.0),
    }
}
