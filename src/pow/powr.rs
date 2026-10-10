//! Powers restricted to exp(y*log(x)).
// Special cases adapted from Arm optimized-routines
// math/aarch64/advsimd/powr.c and powrf.c, with changes.
// The numerical kernels are shared with the SLEEF-derived pow implementation.

/*
 * Copyright (c) 2025-2026, Arm Limited.
 * Copyright (c) 2025, Arm Limited.
 * SPDX-License-Identifier: MIT
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use core::simd::prelude::*;

macro_rules! powr {
    ($function:ident, $kind:ident, $pow:path) => {
        /// Computes exp(y*ln(x)) for each lane.
        /// Negative x, 0^0, infinity^0, 1^infinity, and NaN inputs give NaN.
        #[inline]
        pub fn $function<const N: usize>(x: Simd<$kind, N>, y: Simd<$kind, N>) -> Simd<$kind, N> {
            if (x.simd_gt(Simd::splat(0.0)) & x.is_finite() & y.is_finite()).all() {
                return $pow(x, y);
            }
            let invalid = x.simd_lt(Simd::splat(0.0))
                | x.is_nan()
                | y.is_nan()
                | ((x.simd_eq(Simd::splat(0.0)) | x.is_infinite()) & y.simd_eq(Simd::splat(0.0)))
                | (x.simd_eq(Simd::splat(1.0)) & y.is_infinite());
            invalid.select(Simd::splat($kind::NAN), $pow(x.abs(), y))
        }
    };
}

powr!(powr_f32, f32, super::pow_f32);
powr!(powr_f64, f64, super::pow_f64);
