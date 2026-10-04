/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log10f.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2020-2025, Arm Limited.
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

use super::super::reduction::{finish_f32, normalize_f32};
use core::simd::prelude::*;
use std::simd::StdFloat;

const LOG10F_COEFF: [f32; 8] = [
    f32::from_bits(0xbe5e5bce), // -0x1.bcb79cp-3
    f32::from_bits(0x3e143ce4), // 0x1.2879c8p-3
    f32::from_bits(0xbdde6a39), // -0x1.bcd472p-4
    f32::from_bits(0x3db2047c), // 0x1.6408f8p-4
    f32::from_bits(0xbd9237c0), // -0x1.246f8p-4
    f32::from_bits(0x3d78728a), // 0x1.f0e514p-5
    f32::from_bits(0xbd87e496), // -0x1.0fc92cp-4
    f32::from_bits(0x3d7afbb5), // 0x1.f5f76ap-5
];

#[inline]
fn reduce<const N: usize>(bits: Simd<u32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
    let off = Simd::splat(0x3f2aaaabu32);
    let u = bits - off;
    let n = (u.cast::<i32>() >> 23).cast::<f32>();
    let r = Simd::<f32, N>::from_bits((u & Simd::splat(0x007fffff)) + off) - Simd::splat(1.0);
    (n, r)
}

#[inline]
fn normal_bits<const N: usize>(x: Simd<f32, N>) -> Simd<u32, N> {
    let bits = x.to_bits();
    let special = (bits - Simd::splat(0x00800000)).simd_ge(Simd::splat(0x7f000000));
    if special.any() {
        normalize_f32(x)
    } else {
        bits
    }
}

#[inline]
fn finish<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>) -> Simd<f32, N> {
    let bits = x.to_bits();
    if (bits - Simd::splat(0x00800000))
        .simd_ge(Simd::splat(0x7f000000))
        .any()
    {
        finish_f32(x, y)
    } else {
        y
    }
}

/// Computes the decimal logarithm of each lane.
#[inline]
pub fn log10_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let (n, r) = reduce(normal_bits(x));
    let c = LOG10F_COEFF.map(Simd::splat);
    let r2 = r * r;
    let c01 = r.mul_add(c[1], c[0]);
    let c23 = r.mul_add(c[3], c[2]);
    let c45 = r.mul_add(c[5], c[4]);
    let c67 = r.mul_add(c[7], c[6]);
    let p = r2.mul_add(c67, c45);
    let p = r2.mul_add(p, c23);
    let p = r2.mul_add(p, c01);
    let hi = n.mul_add(Simd::splat(f32::from_bits(0x3f317218)), r);
    let hi = hi * Simd::splat(f32::from_bits(0x3ede5bd9));
    finish(x, r2.mul_add(p, hi))
}
