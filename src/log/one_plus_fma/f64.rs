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
    let c = COEFF_F64.map(Simd::splat);
    let p0_0 = m.mul_add(c[1], c[0]);
    let p0_1 = m.mul_add(c[3], c[2]);
    let p0_2 = m.mul_add(c[5], c[4]);
    let p0_3 = m.mul_add(c[7], c[6]);
    let p0_4 = m.mul_add(c[9], c[8]);
    let p0_5 = m.mul_add(c[11], c[10]);
    let p0_6 = m.mul_add(c[13], c[12]);
    let p0_7 = m.mul_add(c[15], c[14]);
    let p0_8 = m.mul_add(c[17], c[16]);
    let m2 = m * m;
    let p1_0 = m2.mul_add(p0_1, p0_0);
    let p1_1 = m2.mul_add(p0_3, p0_2);
    let p1_2 = m2.mul_add(p0_5, p0_4);
    let p1_3 = m2.mul_add(p0_7, p0_6);
    let p1_4 = m2.mul_add(c[18], p0_8);
    let m4 = m2 * m2;
    let p2_0 = m4.mul_add(p1_1, p1_0);
    let p2_1 = m4.mul_add(p1_3, p1_2);
    let m8 = m4 * m4;
    let p3_0 = m8.mul_add(p2_1, p2_0);
    let m16 = m8 * m8;
    let p4_0 = m16.mul_add(p1_4, p3_0);
    p4_0
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
