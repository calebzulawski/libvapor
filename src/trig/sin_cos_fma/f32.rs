/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/sinf.c and cosf.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2025, Arm Limited.
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

use super::super::sin_cos_non_fma::full_range_sincos_f32 as sincos_f32;
use core::simd::prelude::*;
use std::simd::StdFloat;

const POLY_F32: [f32; 4] = [
    f32::from_bits(0xbe2aaaa4), // -0x1.555548p-3
    f32::from_bits(0x3c0886fa), //  0x1.110df4p-7
    f32::from_bits(0xb94fa175), // -0x1.9f42eap-13
    f32::from_bits(0x362d973b), //  0x1.5b2e76p-19
];

#[inline]
pub(crate) fn sin_cos_f32<const N: usize, const COS: bool>(x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    let slow = ax.simd_ge(Simd::splat(1048576.0)) | !x.is_finite();
    // Keep inactive lanes inside the fast reducer's domain.
    let a = slow.select(Simd::splat(0.0), ax);
    let inv_pi = Simd::splat(f32::from_bits(0x3ea2f983));
    let count = if COS {
        a.mul_add(inv_pi, Simd::splat(0.5))
    } else {
        a * inv_pi
    };
    // The shifted mantissa encodes the rounded integer, including its parity.
    // Either nearest integer is valid at a tie at the polynomial endpoints.
    let shift = Simd::splat(f32::from_bits(0x4b400000));
    let shifted = count + shift;
    let odd = shifted.to_bits() << 31;
    let mut n = shifted - shift;
    if COS {
        n -= Simd::splat(0.5);
    }

    let r = n.mul_add(Simd::splat(-f32::from_bits(0x40490fdb)), a);
    let r = n.mul_add(Simd::splat(-f32::from_bits(0xb3bbbd2e)), r);
    let r = n.mul_add(Simd::splat(-f32::from_bits(0xa7772ced)), r);
    let r2 = r * r;
    let p = r2.mul_add(Simd::splat(POLY_F32[3]), Simd::splat(POLY_F32[2]));
    let p = r2.mul_add(p, Simd::splat(POLY_F32[1]));
    let p = r2.mul_add(p, Simd::splat(POLY_F32[0]));
    // Preserve the upstream evaluation order: sine and cosine group r^3
    // differently, which matters to their error bounds.
    let y = if COS {
        (r2 * r).mul_add(p, r)
    } else {
        (p * r2).mul_add(r, r)
    };
    let sign = if COS {
        odd
    } else {
        odd ^ (x.to_bits() & Simd::splat(1 << 31))
    };
    let y = Simd::from_bits(y.to_bits() ^ sign);
    // These tiny inputs round to x or 1, including sin(-0) = -0.
    let tiny = ax.simd_lt(Simd::splat(f32::from_bits(0x39800000)));
    let y = tiny.select(if COS { Simd::splat(1.0) } else { x }, y);
    if !slow.any() {
        return y;
    }
    let (s, c) = sincos_f32(slow.select(x, Simd::splat(0.0)));
    slow.select(if COS { c } else { s }, y)
}
