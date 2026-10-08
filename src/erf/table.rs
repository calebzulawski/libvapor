//! Taylor expansion at the nearest multiple of 1/128.
// Based on Arm optimized-routines math/aarch64/advsimd/erf.c, with changes.
// Copyright (c) 2023-2025, Arm Limited.
// SPDX-License-Identifier: MIT
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::precision::madd;
use core::simd::prelude::*;
#[path = "centers.rs"]
mod centers;

#[inline]
pub(super) fn erf<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let a = x.abs().simd_min(Simd::splat(6.0));
    // The ulp at 2^45 is 1/128. Addition rounds to an exact table center,
    // and the low bits give a bounded index without a float conversion.
    let shift = Simd::splat(35184372088832.0);
    let rounded = a + shift;
    let index = (rounded.to_bits() - shift.to_bits()).cast::<usize>();
    let r = rounded - shift;
    let (value, derivative) = crate::table::lookup_pairs(&centers::TABLE, index);
    let d = a - r;
    let d2 = d * d;
    let r2 = r * r;
    let p1 = -r;
    let p2 = madd(r2, Simd::splat(2.0 / 3.0), Simd::splat(-1.0 / 3.0));
    let p3 = r * madd(r2, Simd::splat(-1.0 / 3.0), Simd::splat(0.5));
    let p4 = madd(
        r2,
        madd(r2, Simd::splat(2.0 / 15.0), Simd::splat(-0.4)),
        Simd::splat(0.1),
    );
    let p5 = r * madd(
        r2,
        madd(r2, Simd::splat(-2.0 / 45.0), Simd::splat(2.0 / 9.0)),
        Simd::splat(-1.0 / 6.0),
    );
    let p12 = madd(d, p2, p1);
    let p34 = madd(d, p4, p3);
    let p = madd(d2, madd(d2, p5, p34), p12);
    let y = madd(derivative, madd(d2, p, d), value);
    x.is_nan().select(x + x, y.copysign(x))
}
