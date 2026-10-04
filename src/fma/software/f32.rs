/*
 * Derived from musl src/math/fmaf.c and musl v1.1.16 src/math/fma.c,
 * with changes. Copyright and license notices inherited from musl follow.
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

/*
 * Copyright © 2005-2020 Rich Felker, et al.
 * Copyright © 2005-2014 Rich Felker, et al.
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use core::simd::prelude::*;

/// Computes the fused multiply-add x*y+z for each lane.
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
fn correct_halfway<const N: usize>(
    xy: Simd<f64, N>,
    z: Simd<f64, N>,
    result: Simd<f64, N>,
) -> Simd<f32, N> {
    // TwoSum recovers the exact addition residual. The product of two f32
    // values is exact in f64, and neither the product nor this sum can
    // overflow or underflow f64. Rounding the sum to odd prevents double
    // rounding at both normal and subnormal f32 midpoints.
    let s = result - xy;
    let err = (xy - (result - s)) + (z - s);
    let bits = result.to_bits();
    let adjust = result.is_finite()
        & err.simd_ne(Simd::splat(0.0))
        & (bits & Simd::splat(1)).simd_eq(Simd::splat(0));
    let step = Simd::splat(1) - (((bits ^ err.to_bits()) >> 62) & Simd::splat(2));
    Simd::<f64, N>::from_bits(adjust.select(bits + step, bits)).cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_midpoints_round_once() {
        // The product is just below half an ULP at one, but the f64 sum
        // rounds to the midpoint. Odd f32 mantissas expose double rounding.
        let x = 1.0 + f32::EPSILON;
        let y = (1.0 - f32::EPSILON) * (f32::EPSILON / 2.0);
        let z = Simd::from_array([1.0, x, -1.0, -x]);
        let result = fma_f32(Simd::from_array([x, x, -x, -x]), Simd::splat(y), z);
        assert_eq!(
            result.to_array().map(f32::to_bits),
            z.to_array().map(f32::to_bits)
        );
    }

    #[test]
    fn subnormal_correction_keeps_exceptional_lanes() {
        let result = fma_f32(
            Simd::from_array([0.0, -0.0, 1.0, 1.0]),
            Simd::splat(0.0),
            Simd::from_array([
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NAN,
                f32::from_bits(1),
            ]),
        )
        .to_array();
        assert_eq!(result[0], f32::NEG_INFINITY);
        assert_eq!(result[1], f32::INFINITY);
        assert!(result[2].is_nan());
        assert_eq!(result[3].to_bits(), 1);
    }

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
