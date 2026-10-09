/*
 * Derived from musl src/math/roundf.c, with changes.
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

use super::{MODE_CEIL, MODE_FLOOR, MODE_ROUND_TIES_AWAY, MODE_ROUND_TIES_EVEN, MODE_TRUNC};

#[inline]
fn rounded_f32<const N: usize, const MODE: u8>(x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    if super::USE_INT_CONVERSION && (MODE == MODE_TRUNC || MODE == MODE_ROUND_TIES_AWAY) {
        // Larger finite f32 values are already integers. An ordered comparison
        // also excludes infinities and NaNs from the conversion.
        let regular = ax.simd_lt(Simd::splat(8388608.0));
        let a = regular.select(ax, Simd::splat(0.0));
        // SAFETY: every lane is finite and in [0, 2^23), within the i32 range.
        let integer = unsafe { a.to_int_unchecked::<i32>() }.cast::<f32>();
        let y = if MODE == MODE_TRUNC {
            integer
        } else {
            // Compare the exact fractional remainder rather than adding 0.5,
            // which could round a value just below a halfway point upward.
            let up = (a - integer).simd_ge(Simd::splat(0.5));
            integer + up.select(Simd::splat(1.0), Simd::splat(0.0))
        };
        return regular.select(y.copysign(x), x);
    }
    let large = ax.simd_ge(Simd::splat(8388608.0f32)) | !x.is_finite();
    let a = large.select(Simd::splat(0.0), ax);
    let bias = Simd::splat(8388608.0f32);
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
    } else if MODE == MODE_ROUND_TIES_AWAY {
        nearest + (a - nearest).simd_eq(Simd::splat(0.5)).select(one, zero)
    } else {
        // The bias addition already rounds to nearest with ties to even.
        nearest
    };
    large.select(x, y.copysign(x))
}

/// Rounds each lane toward zero.
#[inline]
pub fn trunc_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    rounded_f32::<N, MODE_TRUNC>(x)
}

/// Rounds each lane toward negative infinity.
#[inline]
pub fn floor_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    rounded_f32::<N, MODE_FLOOR>(x)
}

/// Rounds each lane toward positive infinity.
#[inline]
pub fn ceil_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    rounded_f32::<N, MODE_CEIL>(x)
}

/// Rounds each lane to nearest, with ties away from zero.
#[inline]
pub fn round_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    rounded_f32::<N, MODE_ROUND_TIES_AWAY>(x)
}

/// Rounds each lane to nearest, with ties to even.
#[inline]
pub fn roundeven_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    rounded_f32::<N, MODE_ROUND_TIES_EVEN>(x)
}

#[cfg(test)]
mod tests {
    use super::super::tests::rounding_boundaries;
    use super::*;

    // 2^23: every larger finite f32 value is already integral.
    rounding_boundaries!(f32; 8388608.0);
}
