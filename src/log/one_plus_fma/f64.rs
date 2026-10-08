/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log1p.c and v_log1p_inline.h, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2022-2025, Arm Limited.
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
use std::simd::StdFloat;

const COEFF_F64: [f64; 19] = [
    f64::from_bits(0xbfdffffffffffffb), // -0x1.ffffffffffffbp-2
    f64::from_bits(0x3fd55555555551a9), // 0x1.55555555551a9p-2
    f64::from_bits(0xbfd00000000008e3), // -0x1.00000000008e3p-2
    f64::from_bits(0x3fc9999999a32797), // 0x1.9999999a32797p-3
    f64::from_bits(0xbfc555555552fecf), // -0x1.555555552fecfp-3
    f64::from_bits(0x3fc249248e071e5a), // 0x1.249248e071e5ap-3
    f64::from_bits(0xbfbffffff8bf8482), // -0x1.ffffff8bf8482p-4
    f64::from_bits(0x3fbc71c8f07da57a), // 0x1.c71c8f07da57ap-4
    f64::from_bits(0xbfb9999ca4ccb617), // -0x1.9999ca4ccb617p-4
    f64::from_bits(0x3fb7459ad2e1dfa3), // 0x1.7459ad2e1dfa3p-4
    f64::from_bits(0xbfb554d2680a3ff2), // -0x1.554d2680a3ff2p-4
    f64::from_bits(0x3fb3b4c54d487455), // 0x1.3b4c54d487455p-4
    f64::from_bits(0xbfb2548a9ffe80e6), // -0x1.2548a9ffe80e6p-4
    f64::from_bits(0x3fb0f389a24b2e07), // 0x1.0f389a24b2e07p-4
    f64::from_bits(0xbfaeee4db15db335), // -0x1.eee4db15db335p-5
    f64::from_bits(0x3fae95b494d4a5dd), // 0x1.e95b494d4a5ddp-5
    f64::from_bits(0xbfb15fdf07cb7c73), // -0x1.15fdf07cb7c73p-4
    f64::from_bits(0x3fb0310b70800fcf), // 0x1.0310b70800fcfp-4
    f64::from_bits(0xbf9cfa7385bdb37e), // -0x1.cfa7385bdb37ep-6
];

#[inline]
fn poly_f64<const N: usize>(m: Simd<f64, N>) -> Simd<f64, N> {
    // Two interleaved Horner chains keep few vectors live even when N
    // spans several hardware registers.
    let c = |index: usize| Simd::splat(COEFF_F64[index]);
    let m2 = m * m;
    let mut even = c(18);
    let mut odd = c(17);
    for i in (0..=16).rev().step_by(2) {
        even = m2.mul_add(even, c(i));
        if i > 0 {
            odd = m2.mul_add(odd, c(i - 1));
        }
    }
    m.mul_add(odd, even)
}

#[inline]
fn log1p_f64_kernel<const N: usize>(a: Simd<f64, N>) -> Simd<f64, N> {
    let u = a + Simd::splat(1.0);
    let bits = u.to_bits() + Simd::splat(0x00095f6200000000);
    let k = ((bits >> 52).cast::<i32>() - Simd::splat(1023)).cast::<f64>();
    let reduced = (bits & Simd::splat(0x000fffffffffffff)) + Simd::splat(0x3fe6a09e00000000);
    let f = Simd::<f64, N>::from_bits(reduced) - Simd::splat(1.0);
    let correction = (a - (u - Simd::splat(1.0))) / u;
    let k0 = k.simd_eq(Simd::splat(0.0));
    let correction = k0.select(Simd::splat(0.0), correction);
    let f = k0.select(a, f);
    let lo = k.mul_add(Simd::splat(f64::from_bits(0x3d2ef35793c76730)), correction);
    let hi = k.mul_add(Simd::splat(f64::from_bits(0x3fe62e42fefa3800)), f);
    (f * f).mul_add(poly_f64(f), lo + hi)
}

#[cold]
#[inline(never)]
fn log1p_f64_special<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let special = !(x.simd_gt(Simd::splat(-1.0)) & x.simd_lt(Simd::splat(f64::INFINITY)));
    let a = special.select(Simd::splat(0.0), x);
    let y = log1p_f64_kernel(a);
    let y = x.abs().simd_lt(Simd::splat(1.0e-16)).select(x, y);
    let exceptional = x
        .simd_eq(Simd::splat(-1.0))
        .select(Simd::splat(f64::NEG_INFINITY), Simd::splat(f64::NAN));
    let exceptional = x.simd_eq(Simd::splat(f64::INFINITY)).select(x, exceptional);
    let exceptional = x.is_nan().select(x + x, exceptional);
    special.select(exceptional, y)
}

/// Computes log(1+x) using compensated reduction and polynomial approximation.
#[inline]
pub fn log1p_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let regular = x.simd_gt(Simd::splat(-1.0))
        & x.simd_lt(Simd::splat(f64::INFINITY))
        & x.abs().simd_ge(Simd::splat(1.0e-16));
    if regular.all() {
        log1p_f64_kernel(x)
    } else {
        log1p_f64_special(x)
    }
}
