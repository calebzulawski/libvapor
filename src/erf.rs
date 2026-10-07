//! Error functions, including accurate complementary tails.
/*
 * Derived from musl src/math/erf.c and erff.c, with changes.
 * Single-precision conversion by Ian Lance Taylor, Cygnus Support.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Copyright © 2005-2020 Rich Felker, et al.
 * SPDX-License-Identifier: MIT AND SunPro
 *
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 */

mod data;
use crate::precision::{exp_value, madd, madd_f32, Dd};
use core::simd::prelude::*;
use data::*;

#[inline]
fn polynomial<const N: usize>(
    x: Simd<f64, N>,
    count: usize,
    coefficient: impl Fn(usize) -> Simd<f64, N>,
) -> Simd<f64, N> {
    let x2 = x * x;
    let mut even = coefficient(0);
    let mut odd = coefficient(1);
    for i in (2..count).step_by(2) {
        even = madd(even, x2, coefficient(i));
        if i + 1 < count {
            odd = madd(odd, x2, coefficient(i + 1));
        }
    }
    if count % 2 == 0 {
        madd(even, x, odd)
    } else {
        madd(odd, x, even)
    }
}

#[inline]
fn ratio<const N: usize>(x: Simd<f64, N>, p: &[f64], q: &[f64]) -> Simd<f64, N> {
    polynomial(x, p.len(), |i| Simd::splat(p[i])) / polynomial(x, q.len(), |i| Simd::splat(q[i]))
}

#[inline]
fn small_erf<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let z = x * x;
    madd(x, ratio(z, &PP, &QQ), x)
}

#[inline]
fn complementary<const N: usize, const EXTRA_PRECISION: bool>(a: Simd<f64, N>) -> Simd<f64, N> {
    let mid = a.simd_lt(Simd::splat(1.25));
    let s = a - Simd::splat(1.0);
    let middle = Simd::splat(1.0 - ERX) - ratio(s, &PA, &QA);
    if mid.all() {
        return middle;
    }
    let x = a.simd_clamp(Simd::splat(1.25), Simd::splat(28.0));
    let z = Simd::splat(1.0) / (x * x);
    let near = x.simd_lt(Simd::splat(2.857142857142857));
    let numerator = polynomial(z, RA.len(), |i| {
        near.select(Simd::splat(RA[i]), Simd::splat(RB[i]))
    });
    let denominator = polynomial(z, SA.len(), |i| {
        near.select(Simd::splat(SA[i]), Simd::splat(SB[i]))
    });
    // Compensate x*x before exponentiation: its rounded error grows in the tail.
    let exponent = Dd::product(-x, x)
        .add_float(Simd::splat(-0.5625))
        .add_float(numerator / denominator);
    let tail = if EXTRA_PRECISION {
        exp_value(exponent) / x
    } else {
        let exponential = crate::exp_f64(exponent.hi);
        madd(exponential, exponent.lo, exponential) / x
    };
    let tail = a.simd_ge(Simd::splat(28.0)).select(Simd::splat(0.0), tail);
    mid.select(middle, tail)
}

/// Computes the error function for each lane.
#[inline]
pub fn erf_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let small = ax.simd_lt(Simd::splat(0.84375));
    let a = ax.simd_min(Simd::splat(0.84375));
    let near = small_erf(a);
    let value = if small.all() {
        near
    } else {
        let far = Simd::splat(1.0)
            - complementary::<N, false>(ax.simd_clamp(Simd::splat(0.84375), Simd::splat(6.0)));
        let far = ax.simd_ge(Simd::splat(6.0)).select(Simd::splat(1.0), far);
        small.select(near, far)
    };
    x.is_nan().select(x + x, value.copysign(x))
}

/// Computes 1-erf(x) for each lane, retaining accuracy in the positive tail.
#[inline]
pub fn erfc_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    erfc::<N, true>(x)
}

#[inline]
fn erfc<const N: usize, const EXTRA_PRECISION: bool>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let small = ax.simd_lt(Simd::splat(0.84375));
    let a = ax.simd_min(Simd::splat(0.84375));
    let near = Simd::splat(1.0) - small_erf(a);
    let value = if small.all() {
        near
    } else {
        small.select(
            near,
            complementary::<N, EXTRA_PRECISION>(ax.simd_max(Simd::splat(0.84375))),
        )
    };
    let value = x.is_sign_negative().select(Simd::splat(2.0) - value, value);
    x.is_nan().select(x + x, value)
}

/// Computes the error function for each lane.
#[inline]
pub fn erf_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    let small = ax.simd_lt(Simd::splat(0.84375));
    let a = ax.simd_min(Simd::splat(0.84375));
    let z = a * a;
    let near = madd_f32(a, polynomial_f32(z, &PP) / polynomial_f32(z, &QQ), a);
    let value = if small.all() {
        near
    } else {
        let far = Simd::splat(1.0)
            - complementary_f32::<N, false>(ax.simd_clamp(Simd::splat(0.84375), Simd::splat(6.0)));
        small.select(
            near,
            ax.simd_ge(Simd::splat(6.0)).select(Simd::splat(1.0), far),
        )
    };
    x.is_nan().select(x + x, value.copysign(x))
}

/// Computes 1-erf(x) for each lane, retaining accuracy in the positive tail.
#[inline]
pub fn erfc_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    let small = ax.simd_lt(Simd::splat(0.84375));
    let a = ax.simd_min(Simd::splat(0.84375));
    let z = a * a;
    let correction = a * (polynomial_f32(z, &PP) / polynomial_f32(z, &QQ));
    let near = a.simd_lt(Simd::splat(0.25)).select(
        Simd::splat(1.0) - (a + correction),
        Simd::splat(0.5) - ((a - Simd::splat(0.5)) + correction),
    );
    let value = if small.all() {
        near
    } else {
        small.select(
            near,
            complementary_f32::<N, true>(ax.simd_max(Simd::splat(0.84375))),
        )
    };
    let value = x.is_sign_negative().select(Simd::splat(2.0) - value, value);
    x.is_nan().select(x + x, value)
}

#[inline]
fn polynomial_f32<const N: usize>(x: Simd<f32, N>, coefficients: &[f64]) -> Simd<f32, N> {
    let mut p = Simd::splat(coefficients[0] as f32);
    for &c in &coefficients[1..] {
        p = madd_f32(p, x, Simd::splat(c as f32));
    }
    p
}

#[inline]
fn complementary_f32<const N: usize, const PRECISE_TAIL: bool>(a: Simd<f32, N>) -> Simd<f32, N> {
    let mid = a.simd_lt(Simd::splat(1.25));
    let s = a - Simd::splat(1.0);
    let middle = Simd::splat(1.0 - ERX as f32) - polynomial_f32(s, &PA) / polynomial_f32(s, &QA);
    if mid.all() {
        return middle;
    }
    let x = a.simd_clamp(Simd::splat(1.25), Simd::splat(11.0));
    let t = Simd::splat(1.0) / (x * x);
    let near = x.simd_lt(Simd::splat(2.857142857142857));
    let mut numerator = near.select(Simd::splat(RA[0] as f32), Simd::splat(RB[0] as f32));
    let mut denominator = near.select(Simd::splat(SA[0] as f32), Simd::splat(SB[0] as f32));
    for i in 1..RA.len() {
        numerator = madd_f32(
            numerator,
            t,
            near.select(Simd::splat(RA[i] as f32), Simd::splat(RB[i] as f32)),
        );
    }
    for i in 1..SA.len() {
        denominator = madd_f32(
            denominator,
            t,
            near.select(Simd::splat(SA[i] as f32), Simd::splat(SB[i] as f32)),
        );
    }
    // Truncate to eleven significant bits so the main square and the .5625
    // offset are exact; put the discarded square into a second exponential.
    let z = Simd::<f32, N>::from_bits(x.to_bits() & Simd::splat(0xffffe000));
    let correction = crate::exp_f32(madd_f32(z - x, z + x, numerator / denominator));
    let tail = if PRECISE_TAIL {
        let main = crate::exp_f64((-z * z - Simd::splat(0.5625)).cast());
        ((main * correction.cast::<f64>()) / x.cast::<f64>()).cast::<f32>()
    } else {
        let main = crate::exp_f32(-z * z - Simd::splat(0.5625));
        (main * correction) / x
    };
    let tail = a.simd_ge(Simd::splat(11.0)).select(Simd::splat(0.0), tail);
    mid.select(middle, tail)
}
