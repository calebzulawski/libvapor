/*
 * Adapted from musl libc
 * Copyright © 2005-2020 Rich Felker, et al.
 * SPDX-License-Identifier: MIT
 *
 * Copyright (c) 2005-2011 David Schultz <das@FreeBSD.ORG>
 * SPDX-License-Identifier: BSD-2-Clause
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

/// Computes the fused multiply-add x*y+z for each lane.
#[allow(
    unused_parens,
    reason = "vectorize! retains grouping around scalar comparisons"
)]
#[inline]
pub fn fma_f32<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>, z: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let x = x as f64;
        let y = y as f64;
        let z = z as f64;
        let xy = x * y;
        let result = xy + z;

        let halfway = result.to_bits() & 0x1fffffff == 0x10000000;
        let exact = (result - xy == z) & (result - z == xy);
        if !halfway | result.is_nan() | exact {
            result as f32
        } else {
            let err = if result.is_sign_negative() == (z > xy) {
                xy - result + z
            } else {
                z - result + xy
            };
            if result.is_sign_negative() == (err < 0.0) {
                <f64>::from_bits(result.to_bits() + 1) as f32
            } else {
                <f64>::from_bits(result.to_bits() - 1) as f32
            }
        }
    })
}
