/*
 * Derived from musl src/math/__rem_pio2.c, with changes.
 * SPDX-License-Identifier: MIT AND SunPro
 */

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 *
 * Optimized by Bruce D. Evans.
 */

use super::super::data::*;
use super::{round_quadrant, Reduced};
use core::simd::prelude::*;
use simd_macros::vectorize;

pub(super) fn reduce_medium_f64<const N: usize>(ax: Simd<f64, N>) -> Reduced<N> {
    let (quadrant, kd, mut r, mut w, ex, second) = vectorize!(N, {
        let (quadrant, kd) = round_quadrant(ax);
        let r = ax - kd * scalar!(PIO2_1);
        let w = kd * scalar!(PIO2_1T);
        let y = r - w;
        let ex = (ax.to_bits() >> 52) & 0x7ff;
        let ex = ex as i64;
        let ey = (y.to_bits() >> 52) & 0x7ff;
        let ey = ey as i64;
        let cancellation = ex - ey;
        (quadrant, kd, r, w, ex, cancellation > 16)
    });

    if second.any() {
        // Extra chunks of pi/2 recover precision lost near its multiples.
        let (mut r2, mut w2, third) = vectorize!(N, {
            let t = r;
            let w = kd * scalar!(PIO2_2);
            let r = t - w;
            let w = kd * scalar!(PIO2_2T) - ((t - r) - w);
            let y = r - w;
            let ey = (y.to_bits() >> 52) & 0x7ff;
            let ey = ey as i64;
            let cancellation = ex - ey;
            let third = cancellation > 49;
            (r, w, second & third)
        });
        if third.any() {
            let (r3, w3) = vectorize!(N, {
                let t = r2;
                let w = kd * scalar!(PIO2_3);
                let r = t - w;
                let w = kd * scalar!(PIO2_3T) - ((t - r) - w);
                (r, w)
            });
            r2 = third.select(r3, r2);
            w2 = third.select(w3, w2);
        }
        r = second.select(r2, r);
        w = second.select(w2, w);
    }
    let hi = r - w;
    Reduced {
        quadrant,
        hi,
        lo: (r - hi) - w,
    }
}
