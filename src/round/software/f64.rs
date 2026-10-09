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

use super::{MODE_CEIL, MODE_FLOOR, MODE_ROUND_TIES_AWAY, MODE_TRUNC};

#[inline]
fn rounded_f64<const N: usize, const MODE: u8>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let convert = super::USE_INT_CONVERSION && (MODE == MODE_TRUNC || MODE == MODE_ROUND_TIES_AWAY);
    if convert && ax.simd_lt(Simd::splat(2147483648.0)).all() {
        // SAFETY: the ordered comparison proves that all lanes are finite
        // and in [0, 2^31). Truncation therefore produces a representable i32.
        let integer = unsafe { ax.to_int_unchecked::<i32>() }.cast::<f64>();
        let y = if MODE == MODE_TRUNC {
            integer
        } else {
            // Subtraction preserves the fractional remainder, including the
            // neighbors of halfway points. The final float can equal 2^31.
            let up = (ax - integer).simd_ge(Simd::splat(0.5));
            integer + up.select(Simd::splat(1.0), Simd::splat(0.0))
        };
        return y.copysign(x);
    }
    let large = if convert {
        // One ordered comparison handles large values, infinities and NaNs.
        !ax.simd_lt(Simd::splat(4503599627370496.0))
    } else {
        ax.simd_ge(Simd::splat(4503599627370496.0f64)) | !x.is_finite()
    };
    let a = large.select(Simd::splat(0.0), ax);
    let bias = Simd::splat(4503599627370496.0f64);
    let nearest = (a + bias) - bias;
    let down = nearest.simd_gt(a);
    let zero = Simd::splat(0.0);
    let one = Simd::splat(1.0);
    let y = if MODE == MODE_TRUNC {
        nearest - down.select(one, zero)
    } else if MODE == MODE_FLOOR {
        let signed = nearest.copysign(x);
        // Subtracting +0 preserves -0, so this needs only one
        // comparison and no sign-dependent adjustment masks.
        return large.select(x, signed - signed.simd_gt(x).select(one, zero));
    } else if MODE == MODE_CEIL {
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
    rounded_f64::<N, MODE_TRUNC>(x)
}

/// Rounds each lane toward negative infinity.
#[inline]
pub fn floor_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, MODE_FLOOR>(x)
}

/// Rounds each lane toward positive infinity.
#[inline]
pub fn ceil_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, MODE_CEIL>(x)
}

/// Rounds each lane to nearest, with ties away from zero.
#[inline]
pub fn round_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    rounded_f64::<N, MODE_ROUND_TIES_AWAY>(x)
}

#[cfg(test)]
mod tests {
    use super::super::tests::rounding_boundaries;
    use super::*;

    // i32 conversion limits (including a rounded result of 2^31), then 2^52,
    // above which every finite f64 value is already integral.
    rounding_boundaries!(f64;
        2147483647.0,
        2147483648.0 - 1.5,
        2147483648.0 - 0.5,
        2147483648.0,
        4503599627370496.0,
    );
}
