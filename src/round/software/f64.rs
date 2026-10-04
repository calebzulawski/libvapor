/*
 * Derived from musl src/math/round.c, with changes.
 */

/*
 * Copyright © 2005-2020 Rich Felker, et al.
 * SPDX-License-Identifier: MIT
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

#[inline]
fn rounded_f64<const N: usize, const MODE: u8>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let large = ax.simd_ge(Simd::splat(4503599627370496.0f64)) | !x.is_finite();
    let a = large.select(Simd::splat(0.0), ax);
    let bias = Simd::splat(4503599627370496.0f64);
    let nearest = (a + bias) - bias;
    let down = nearest.simd_gt(a);
    let zero = Simd::splat(0.0);
    let one = Simd::splat(1.0);
    let y = if MODE == 0 {
        nearest - down.select(one, zero)
    } else if MODE == 1 {
        let signed = nearest.copysign(x);
        // Subtracting +0 preserves -0, so this needs only one
        // comparison and no sign-dependent adjustment masks.
        return large.select(x, signed - signed.simd_gt(x).select(one, zero));
    } else if MODE == 2 {
        let signed = nearest.copysign(x);
        // ceil(x) = -floor(-x). Keep the subtraction form to
        // preserve negative zero when a negative input rounds up.
        return large.select(x, -(-signed - signed.simd_lt(x).select(one, zero)));
    } else {
        nearest + (a - nearest).simd_eq(Simd::splat(0.5)).select(one, zero)
    };
    large.select(x, y.copysign(x))
}

/// Rounds each lane toward zero.
#[inline]
pub fn trunc_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, 0>(x)
}

/// Rounds each lane toward negative infinity.
#[inline]
pub fn floor_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, 1>(x)
}

/// Rounds each lane toward positive infinity.
#[inline]
pub fn ceil_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, 2>(x)
}

/// Rounds each lane to nearest, with ties away from zero.
#[inline]
pub fn round_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, 3>(x)
}

/// Computes the fractional part of each lane.
#[inline]
pub fn fract_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    x - trunc_f64(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_halfway_and_neighbors() {
        let below = 0.5f64.next_down();
        let above = 0.5f64.next_up();
        let x = Simd::from_array([below, 0.5, above, 2.5, -below, -0.5, -above, -2.5]);
        let expected = [0.0f64, 1.0, 1.0, 3.0, -0.0, -1.0, -1.0, -3.0];
        assert_eq!(
            round_f64(x).to_array().map(f64::to_bits),
            expected.map(f64::to_bits)
        );
    }

    #[test]
    fn rounding_at_integer_precision_boundary() {
        let boundary = 4503599627370496.0f64; // 2^52: all larger f64 values are integers.
        let x = Simd::from_array([
            boundary.next_down(),
            boundary,
            boundary.next_up(),
            -boundary.next_down(),
            -boundary,
            -boundary.next_up(),
            0.75,
            -0.75,
        ]);
        let input = x.to_array();
        for (name, result, expected) in [
            ("trunc", trunc_f64(x), input.map(f64::trunc)),
            ("floor", floor_f64(x), input.map(f64::floor)),
            ("ceil", ceil_f64(x), input.map(f64::ceil)),
            ("round", round_f64(x), input.map(f64::round)),
            ("fract", fract_f64(x), input.map(f64::fract)),
        ] {
            assert_eq!(
                result.to_array().map(f64::to_bits),
                expected.map(f64::to_bits),
                "{name}"
            );
        }
    }
}
