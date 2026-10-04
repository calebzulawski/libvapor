/*
 * Derived from musl src/math/sqrtf.c, with changes.
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

use super::table::RSQRT_TAB;
use core::simd::prelude::*;
use simd_macros::vectorize;

/* returns a*b*2^-32 - e, with error 0 <= e < 1.  */
fn mul32<const N: usize>(a: Simd<u32, N>, b: Simd<u32, N>) -> Simd<u32, N> {
    ((a.cast::<u64>() * b.cast::<u64>()) >> 32).cast()
}

/// Computes the correctly rounded square root without a square-root instruction.
/// Assumes round-to-nearest, ties-to-even.
#[inline]
pub fn sqrt_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let regular = x.simd_ge(Simd::splat(f32::MIN_POSITIVE)) & x.simd_lt(Simd::splat(f32::INFINITY));
    if regular.all() {
        sqrt_f32_normal(x)
    } else {
        sqrt_f32_general(x)
    }
}

#[cold]
#[inline(never)]
fn sqrt_f32_general<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        if (x == scalar!(f32::INFINITY)) | (x == 0.0) {
            x
        } else if x.is_nan() | (x < 0.0) {
            scalar!(f32::NAN)
        } else {
            let x1p23 = scalar!(f32::from_bits(0x4b000000));
            let x = if x.is_subnormal() {
                <f32>::from_bits((x * x1p23).to_bits() - (23u32 << 23))
            } else {
                x
            };

            let even = x.to_bits() & 0x00800000 != 0;
            let m = if even {
                (x.to_bits() << 7) & 0x7fffffff
            } else {
                (x.to_bits() << 8) | 0x80000000
            };

            let mut ey = x.to_bits() >> 1;
            ey += 0x3f800000u32 >> 1;
            ey &= 0x7f800000;

            let three = 0xc0000000;
            let i = (x.to_bits() >> 17) % 128;
            let mut r = <u32>::gather_or(&RSQRT_TAB, i as usize, 0) << 16;
            let mut s = mul32(m, r);
            let mut d = mul32(s, r);
            let mut u = three - d;
            r = mul32(r, u) << 1;
            s = mul32(s, u) << 1;
            d = mul32(s, r);
            u = three - d;
            s = mul32(s, u);
            s = (s - 1) >> 6;

            let d0 = (m << 16) - s * s;
            let d1 = s - d0;
            let d2 = d1 + s + 1;
            s += d1 >> 31;
            s &= 0x007fffff;
            s |= ey;
            let y = <f32>::from_bits(s);

            let mut tiny = if d2 == 0 { 0 } else { 0x01000000 };
            tiny |= (d1 ^ d2) & 0x80000000;
            y + <f32>::from_bits(tiny)
        }
    })
}

#[inline]
fn sqrt_f32_normal<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let even = x.to_bits() & 0x00800000 != 0;
        let m = if even {
            (x.to_bits() << 7) & 0x7fffffff
        } else {
            (x.to_bits() << 8) | 0x80000000
        };

        let mut ey = x.to_bits() >> 1;
        ey += 0x3f800000u32 >> 1;
        ey &= 0x7f800000;

        let three = 0xc0000000;
        let i = (x.to_bits() >> 17) % 128;
        let mut r = <u32>::gather_or(&RSQRT_TAB, i as usize, 0) << 16;
        let mut s = mul32(m, r);
        let mut d = mul32(s, r);
        let mut u = three - d;
        r = mul32(r, u) << 1;
        s = mul32(s, u) << 1;
        d = mul32(s, r);
        u = three - d;
        s = mul32(s, u);
        s = (s - 1) >> 6;

        let d0 = (m << 16) - s * s;
        let d1 = s - d0;
        let d2 = d1 + s + 1;
        s += d1 >> 31;
        s &= 0x007fffff;
        s |= ey;
        let y = <f32>::from_bits(s);

        let mut tiny = if d2 == 0 { 0 } else { 0x01000000 };
        tiny |= (d1 ^ d2) & 0x80000000;
        y + <f32>::from_bits(tiny)
    })
}
