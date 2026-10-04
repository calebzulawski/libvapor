/*
 * Derived from musl src/math/sqrt.c, with changes.
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

/* returns a*b*2^-64 - e, with error 0 <= e < 3.  */
fn mul64<const N: usize>(a: Simd<u64, N>, b: Simd<u64, N>) -> Simd<u64, N> {
    let ahi = a >> 32;
    let alo = a & Simd::splat(0xffffffff);
    let bhi = b >> 32;
    let blo = b & Simd::splat(0xffffffff);
    ahi * bhi + (ahi * blo >> 32) + (alo * bhi >> 32)
}

/// Computes the correctly rounded square root without a square-root instruction.
/// Assumes round-to-nearest, ties-to-even.
#[inline]
pub fn sqrt_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let regular = x.simd_ge(Simd::splat(f64::MIN_POSITIVE)) & x.simd_lt(Simd::splat(f64::INFINITY));
    if regular.all() {
        sqrt_f64_normal(x)
    } else {
        sqrt_f64_general(x)
    }
}

#[cold]
#[inline(never)]
#[allow(
    unused_braces,
    unused_parens,
    reason = "vectorize! retains scalar branch braces and grouping"
)]
fn sqrt_f64_general<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        if (x == scalar!(f64::INFINITY)) | (x == 0.0) {
            x
        } else if x.is_nan() | (x < 0.0) {
            scalar!(f64::NAN)
        } else {
            let subnormal = x.is_subnormal();
            let x = if subnormal {
                let x1p52 = scalar!(f64::from_bits(0x4330000000000000));
                x * x1p52
            } else {
                x
            };
            let top = if subnormal {
                (x.to_bits() >> 52) - 52
            } else {
                x.to_bits() >> 52
            };

            let even = (top & 1) != 0;
            let m = (x.to_bits() << 11) | 0x8000000000000000;
            let m = if even { m >> 1 } else { m };
            let top = (top + 0x3ff) >> 1;

            let three32 = scalar!(0xc0000000u32);
            let i = (x.to_bits() >> 46) % 128;
            let mut r32 = <u32>::gather_or(&RSQRT_TAB, i as usize, 0) << 16;
            let mut s32 = mul32((m >> 32) as u32, r32);
            let mut d32 = mul32(s32, r32);
            let mut u32 = three32 - d32;
            r32 = mul32(r32, u32) << 1;
            s32 = mul32(s32, u32) << 1;
            d32 = mul32(s32, r32);
            u32 = three32 - d32;
            r32 = mul32(r32, u32) << 1;
            let r = (r32 as u64) << 32;
            let mut s: u64;
            let d: u64;
            let u: u64;
            let three = scalar!(0xc0000000u64);
            s = mul64(m, r);
            d = mul64(s, r);
            u = (three << 32) - d;
            s = mul64(s, u);
            s = (s - 2) >> 9;

            let d0 = (m << 42) - s * s;
            let d1 = s - d0;
            let d2 = d1 + s + 1;
            s += d1 >> 63;
            s &= 0x000fffffffffffff;
            s |= top << 52;
            let y = <f64>::from_bits(s);

            let mut tiny = if d2 == 0 { 0 } else { 0x0010000000000000 };
            tiny |= (d1 ^ d2) & 0x8000000000000000;
            y + <f64>::from_bits(tiny)
        }
    })
}

#[inline]
#[allow(
    unused_braces,
    unused_parens,
    reason = "vectorize! retains scalar branch braces and grouping"
)]
fn sqrt_f64_normal<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let top = x.to_bits() >> 52;
        let even = (top & 1) != 0;
        let m = (x.to_bits() << 11) | 0x8000000000000000;
        let m = if even { m >> 1 } else { m };
        let top = (top + 0x3ff) >> 1;

        let three32 = scalar!(0xc0000000u32);
        let i = (x.to_bits() >> 46) % 128;
        let mut r32 = <u32>::gather_or(&RSQRT_TAB, i as usize, 0) << 16;
        let mut s32 = mul32((m >> 32) as u32, r32);
        let mut d32 = mul32(s32, r32);
        let mut u32 = three32 - d32;
        r32 = mul32(r32, u32) << 1;
        s32 = mul32(s32, u32) << 1;
        d32 = mul32(s32, r32);
        u32 = three32 - d32;
        r32 = mul32(r32, u32) << 1;
        let r = (r32 as u64) << 32;
        let mut s: u64;
        let d: u64;
        let u: u64;
        let three = scalar!(0xc0000000u64);
        s = mul64(m, r);
        d = mul64(s, r);
        u = (three << 32) - d;
        s = mul64(s, u);
        s = (s - 2) >> 9;

        let d0 = (m << 42) - s * s;
        let d1 = s - d0;
        let d2 = d1 + s + 1;
        s += d1 >> 63;
        s &= 0x000fffffffffffff;
        s |= top << 52;
        let y = <f64>::from_bits(s);

        let mut tiny = if d2 == 0 { 0 } else { 0x0010000000000000 };
        tiny |= (d1 ^ d2) & 0x8000000000000000;
        y + <f64>::from_bits(tiny)
    })
}
