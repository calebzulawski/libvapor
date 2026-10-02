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

pub(super) struct Reduced<const N: usize> {
    pub(super) quadrant: Simd<u64, N>,
    pub(super) hi: Simd<f64, N>,
    pub(super) lo: Simd<f64, N>,
}

fn reduce_large_f32<const N: usize>(bits: Simd<u64, N>) -> Reduced<N> {
    let (quadrant, hi) = vectorize!(N, {
        // Reconstruct the fractional part of x*2/pi with a 32x96-bit product.
        // Each lane gathers its own window of the 192-bit 4/pi table.
        let index = (bits >> 26) & 15;
        let index = index as usize;
        let shift = (bits >> 23) & 7;
        let xi = ((bits & 0xffffff) | 0x800000) << shift;
        let res0 = xi * (<u32>::gather_or(&INV_PIO4_F32, index, 0) as u64);
        let res1 = xi * (<u32>::gather_or(&INV_PIO4_F32, index + 4, 0) as u64);
        let res2 = xi * (<u32>::gather_or(&INV_PIO4_F32, index + 8, 0) as u64);
        let res0 = (res2 >> 32) | (res0 << 32);
        let res0 = res0 + res1;
        let quadrant = (res0 + (1u64 << 61)) >> 62;
        let res0 = res0 - (quadrant << 62);
        let hi = res0 as i64 as f64 * scalar!(PIO2_FIXED_F32);
        (quadrant, hi)
    });
    Reduced {
        quadrant,
        hi,
        lo: Simd::splat(0.0),
    }
}

#[allow(unused_braces)]
pub(super) fn reduce_f32<const N: usize>(ax: Simd<f64, N>, bits: Simd<u64, N>) -> Reduced<N> {
    let large = ax.simd_ge(Simd::splat(REDUCTION_LIMIT_F32));
    let (quadrant, hi) = vectorize!(N, {
        let ax = if large { 0.0 } else { ax };
        let kd = ax * scalar!(INV_PIO2) + scalar!(SHIFT);
        let kd = kd - scalar!(SHIFT);
        let hi = ax - kd * scalar!(PIO2_REDUCE_HI_F32) - kd * scalar!(PIO2_REDUCE_LO_F32);
        (kd as u64, hi)
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

fn reduce_medium_f64<const N: usize>(ax: Simd<f64, N>) -> Reduced<N> {
    let (kd, mut r, mut w, ex, second) = vectorize!(N, {
        let kd = ax * scalar!(INV_PIO2) + scalar!(SHIFT);
        let kd = kd - scalar!(SHIFT);
        let r = ax - kd * scalar!(PIO2_1);
        let w = kd * scalar!(PIO2_1T);
        let y = r - w;
        let ex = (ax.to_bits() >> 52) & 0x7ff;
        let ex = ex as i64;
        let ey = (y.to_bits() >> 52) & 0x7ff;
        let ey = ey as i64;
        let cancellation = ex - ey;
        (kd, r, w, ex, cancellation > 16)
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
        quadrant: kd.cast(),
        hi,
        lo: (r - hi) - w,
    }
}

#[allow(unused_braces)]
fn reduce_large_f64<const N: usize>(ax: Simd<f64, N>) -> Reduced<N> {
    let (index, shift, m0, m1) = vectorize!(N, {
        let bits = ax.to_bits();
        let exponent = (bits >> 52) - 1023;
        let mantissa = (bits & 0x000fffffffffffff) | 0x0010000000000000;
        // x = mantissa * 2^(exponent-52). Retain 256 fractional bits of
        // x*2/pi; the table truncation contributes less than 2^-203.
        let end = exponent + 204;
        let index = (end >> 5) + 2;
        let shift = end & 31;
        (index, shift, mantissa & 0xffffffff, mantissa >> 32)
    });

    // Multiply an exponent-selected 288-bit table window by the two-word
    // significand. Each product fits u64; carrying in base 2^32 is exact.
    let mut words = [Simd::splat(0u64); 9];
    let mut carry = Simd::splat(0u64);
    let mut previous = Simd::splat(0u64);
    for (i, word) in words.iter_mut().enumerate() {
        let (product, next_carry, current) = vectorize!(N, {
            let j = index - scalar!(i as u64);
            let j = j as usize;
            let a = <u32>::gather_or(&TWO_OVER_PI_F64, j, 0) as u64;
            let b = <u32>::gather_or(&TWO_OVER_PI_F64, j + 1, 0) as u64;
            let current = ((a << shift) | (b >> (32 - shift))) & 0xffffffff;
            let p0 = current * m0;
            let p1 = previous * m1;
            let sum = (p0 & 0xffffffff) + (p1 & 0xffffffff) + carry;
            let next_carry = (p0 >> 32) + (p1 >> 32) + (sum >> 32);
            (sum & 0xffffffff, next_carry, current)
        });
        *word = product;
        carry = next_carry;
        previous = current;
    }
    let round_up = words[7].simd_ge(Simd::splat(0x80000000));
    let quadrant = (words[8] + round_up.select(Simd::splat(1), Simd::splat(0))) & Simd::splat(3);

    // When rounding upward, complement the fractional integer before any
    // float conversion. This avoids cancellation in 1 - fraction.
    let mut carry = Simd::splat(1u64);
    for word in &mut words[..8] {
        let current = *word;
        let (magnitude, next_carry) = vectorize!(N, {
            let complement = (current ^ 0xffffffff) + carry;
            let magnitude = if round_up {
                complement & 0xffffffff
            } else {
                current
            };
            (magnitude, complement >> 32)
        });
        *word = magnitude;
        carry = next_carry;
    }

    // Sum the fixed-point magnitude into a double-double fraction, retaining
    // the rounding residual of each addition, including near-zero remainders.
    let mut hi = Simd::splat(0.0);
    let mut lo = Simd::splat(0.0);
    for i in (0..8).rev() {
        let word = words[i];
        let scale = f64::from_bits((1023 - 32 * (8 - i) as u64) << 52);
        (hi, lo) = vectorize!(N, {
            let term = word as f64 * scalar!(scale);
            let sum = hi + term;
            (sum, lo + (term - (sum - hi)))
        });
    }
    let (hi, lo) = vectorize!(N, {
        let fraction = hi + lo;
        let fraction_lo = (hi - fraction) + lo;

        // Dekker's product residual preserves the correction term without
        // requiring hardware FMA or a scalar math-library call.
        let split = 134217729.0 * fraction;
        let a_hi = split - (split - fraction);
        let a_lo = fraction - a_hi;
        let product = fraction * scalar!(PIO2);
        let error = ((a_hi * scalar!(PIO2_SPLIT_HI) - product)
            + a_hi * scalar!(PIO2_SPLIT_LO)
            + a_lo * scalar!(PIO2_SPLIT_HI))
            + a_lo * scalar!(PIO2_SPLIT_LO);
        let tail = fraction_lo * scalar!(PIO2) + fraction * scalar!(PIO2_TAIL) + error;
        let hi = product + tail;
        let lo = (product - hi) + tail;
        let hi = if round_up { -hi } else { hi };
        let lo = if round_up { -lo } else { lo };
        (hi, lo)
    });
    Reduced { quadrant, hi, lo }
}

pub(super) fn reduce_f64<const N: usize>(ax: Simd<f64, N>) -> Reduced<N> {
    let large = ax.simd_ge(Simd::splat(REDUCTION_LIMIT_F64));
    let medium = reduce_medium_f64(large.select(Simd::splat(0.0), ax));
    if !large.any() {
        return medium;
    }
    // Exponent extraction and table indices require finite normal inputs.
    let reduced = reduce_large_f64(large.select(ax, Simd::splat(1048576.0)));
    Reduced {
        quadrant: large.select(reduced.quadrant, medium.quadrant),
        hi: large.select(reduced.hi, medium.hi),
        lo: large.select(reduced.lo, medium.lo),
    }
}
