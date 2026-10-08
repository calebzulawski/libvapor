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
fn sqrt_f64_general<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    (x.simd_eq(Simd::splat(f64::INFINITY)) | x.simd_eq(Simd::splat(0.0))).select(
        x,
        (x.is_nan() | x.simd_lt(Simd::splat(0.0))).select(Simd::splat(f64::NAN), {
            let subnormal = x.is_subnormal();
            let x = subnormal.select(
                {
                    let x1p52 = Simd::splat(f64::from_bits(0x4330000000000000));
                    x * x1p52
                },
                x,
            );
            let top = subnormal.select((x.to_bits() >> 52) - Simd::splat(52), x.to_bits() >> 52);

            let even = (top & Simd::splat(1)).simd_ne(Simd::splat(0));
            let m = (x.to_bits() << 11) | Simd::splat(0x8000000000000000);
            let m = even.select(m >> 1, m);
            let top = (top + Simd::splat(0x3ff)) >> 1;

            let three32 = Simd::splat(0xc0000000u32);
            let i = (x.to_bits() >> 46) % Simd::splat(128);
            let mut r32 =
                Simd::<u32, N>::gather_or(&RSQRT_TAB, i.cast::<usize>(), Simd::splat(0)) << 16;
            let mut s32 = mul32((m >> 32).cast::<u32>(), r32);
            let mut d32 = mul32(s32, r32);
            let mut u32 = three32 - d32;
            r32 = mul32(r32, u32) << 1;
            s32 = mul32(s32, u32) << 1;
            d32 = mul32(s32, r32);
            u32 = three32 - d32;
            r32 = mul32(r32, u32) << 1;
            let r = r32.cast::<u64>() << 32;
            let mut s: Simd<u64, N>;
            let d: Simd<u64, N>;
            let u: Simd<u64, N>;
            let three = Simd::splat(0xc0000000u64);
            s = mul64(m, r);
            d = mul64(s, r);
            u = (three << 32) - d;
            s = mul64(s, u);
            s = (s - Simd::splat(2)) >> 9;

            let d0 = (m << 42) - s * s;
            let d1 = s - d0;
            let d2 = d1 + s + Simd::splat(1);
            s += d1 >> 63;
            s &= Simd::splat(0x000fffffffffffff);
            s |= top << 52;
            let y = Simd::<f64, N>::from_bits(s);

            let mut tiny = d2
                .simd_eq(Simd::splat(0))
                .select(Simd::splat(0), Simd::splat(0x0010000000000000));
            tiny |= (d1 ^ d2) & Simd::splat(0x8000000000000000);
            y + Simd::<f64, N>::from_bits(tiny)
        }),
    )
}

#[inline]
fn sqrt_f64_normal<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let top = x.to_bits() >> 52;
    let even = (top & Simd::splat(1)).simd_ne(Simd::splat(0));
    let m = (x.to_bits() << 11) | Simd::splat(0x8000000000000000);
    let m = even.select(m >> 1, m);
    let top = (top + Simd::splat(0x3ff)) >> 1;

    let three32 = Simd::splat(0xc0000000u32);
    let i = (x.to_bits() >> 46) % Simd::splat(128);
    let mut r32 = Simd::<u32, N>::gather_or(&RSQRT_TAB, i.cast::<usize>(), Simd::splat(0)) << 16;
    let mut s32 = mul32((m >> 32).cast::<u32>(), r32);
    let mut d32 = mul32(s32, r32);
    let mut u32 = three32 - d32;
    r32 = mul32(r32, u32) << 1;
    s32 = mul32(s32, u32) << 1;
    d32 = mul32(s32, r32);
    u32 = three32 - d32;
    r32 = mul32(r32, u32) << 1;
    let r = r32.cast::<u64>() << 32;
    let mut s: Simd<u64, N>;
    let d: Simd<u64, N>;
    let u: Simd<u64, N>;
    let three = Simd::splat(0xc0000000u64);
    s = mul64(m, r);
    d = mul64(s, r);
    u = (three << 32) - d;
    s = mul64(s, u);
    s = (s - Simd::splat(2)) >> 9;

    let d0 = (m << 42) - s * s;
    let d1 = s - d0;
    let d2 = d1 + s + Simd::splat(1);
    s += d1 >> 63;
    s &= Simd::splat(0x000fffffffffffff);
    s |= top << 52;
    let y = Simd::<f64, N>::from_bits(s);

    let mut tiny = d2
        .simd_eq(Simd::splat(0))
        .select(Simd::splat(0), Simd::splat(0x0010000000000000));
    tiny |= (d1 ^ d2) & Simd::splat(0x8000000000000000);
    y + Simd::<f64, N>::from_bits(tiny)
}
