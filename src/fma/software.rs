/*
 * Derived from musl src/math/fmaf.c, with changes.
 * SPDX-License-Identifier: MIT AND BSD-2-Clause
 */

/*
 * Copyright (c) 2005-2011 David Schultz <das@FreeBSD.ORG>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
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
    let xd = x.cast::<f64>();
    let yd = y.cast::<f64>();
    let zd = z.cast::<f64>();
    let xy = xd * yd;
    let result = xy + zd;
    let halfway = (result.to_bits() & Simd::splat(0x1fffffff)).simd_eq(Simd::splat(0x10000000));
    let tiny = result.abs().simd_lt(Simd::splat(f32::MIN_POSITIVE as f64))
        & result
            .abs()
            .simd_ge(Simd::splat(f64::from_bits(0x3690000000000000)));
    if !(halfway | tiny).any() {
        return result.cast();
    }
    correct_halfway(xy, zd, result)
}

#[cold]
#[inline(never)]
#[allow(unused_parens, reason = "vectorize! retains comparison grouping")]
fn correct_halfway<const N: usize>(
    xy: Simd<f64, N>,
    z: Simd<f64, N>,
    result: Simd<f64, N>,
) -> Simd<f32, N> {
    vectorize!(N, {
        // Normal f32 values have 29 discarded double mantissa bits. Subnormal
        // f32 spacing stays at 2^-149, so its midpoint bit moves with the
        // double exponent. Include the implicit bit at the zero midpoint.
        let exponent = (result.to_bits() >> 52) & 0x7ff;
        let subnormal = (exponent >= 873) & (exponent < 897);
        let midpoint_shift = if subnormal { 925 - exponent } else { 28 };
        let midpoint = 1u64 << midpoint_shift;
        let mask = (midpoint << 1) - 1;
        let mantissa = (result.to_bits() & 0x000fffffffffffff) | (1u64 << 52);
        let halfway = (mantissa & mask) == midpoint;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subnormal_midpoints_round_once() {
        // (1 + 2^-23)(1 - 2^-23) * 2^-150 is just below half a
        // subnormal ULP. Adding it must leave z unchanged, even when
        // the intermediate f64 sum rounds to an exact midpoint.
        let x = f32::from_bits(0x1a000001);
        let y = f32::from_bits(0x19fffffe);
        let bits = [0, 1, 2, 3, 0x00200001, 0x00400001, 0x007fffff, 0x00800000];
        for sign in [0, 0x80000000] {
            let z = Simd::from_array(bits.map(|bits| f32::from_bits(bits | sign)));
            let result = fma_f32(
                Simd::splat(f32::from_bits(x.to_bits() | sign)),
                Simd::splat(y),
                z,
            );
            assert_eq!(
                result.to_array().map(f32::to_bits),
                z.to_array().map(f32::to_bits)
            );
        }
    }

    #[test]
    fn cancellation_keeps_product_residual() {
        let x = f32::from_bits(0x3f800001); // 1 + 2^-23
        let y = f32::from_bits(0x3f7ffffe); // 1 - 2^-23
        let result = fma_f32(
            Simd::from_array([x, -x]),
            Simd::splat(y),
            Simd::from_array([-1.0, 1.0]),
        );
        let residual = f32::EPSILON * f32::EPSILON;
        assert_eq!(result.to_array(), [-residual, residual]);
    }

    #[test]
    fn addition_cancels_product_overflow() {
        let x = Simd::from_array([f32::MAX, -f32::MAX]);
        let result = fma_f32(x, Simd::splat(2.0), -x);
        assert_eq!(result.to_array(), x.to_array());
    }
}
