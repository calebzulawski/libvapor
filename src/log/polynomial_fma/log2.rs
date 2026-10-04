/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log2f.c, with changes.
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

use super::super::reduction::{finish_f32, normalize_f32};
use core::simd::prelude::*;
use std::simd::StdFloat;

const LOG2F_COEFF: [f32; 9] = [
    f32::from_bits(0x3fb8aa3b), // 0x1.715476p0
    f32::from_bits(0xbf38aa2c), // -0x1.715458p-1
    f32::from_bits(0x3ef6380e), // 0x1.ec701cp-2
    f32::from_bits(0xbeb8b8d2), // -0x1.7171a4p-2
    f32::from_bits(0x3e93d05c), // 0x1.27a0b8p-2
    f32::from_bits(0xbe728a1f), // -0x1.e5143ep-3
    f32::from_bits(0x3e4ec765), // 0x1.9d8ecap-3
    f32::from_bits(0xbe633ad8), // -0x1.c675bp-3
    f32::from_bits(0x3e4f24a8), // 0x1.9e495p-3
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

/// Computes the binary logarithm of each lane.
#[inline]
pub fn log2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let (n, r) = reduce(normal_bits(x));
    let c = LOG2F_COEFF.map(Simd::splat);
    let r2 = r * r;
    let c01 = r.mul_add(c[1], c[0]);
    let c23 = r.mul_add(c[3], c[2]);
    let c45 = r.mul_add(c[5], c[4]);
    let c67 = r.mul_add(c[7], c[6]);
    let p = r2.mul_add(c[8], c67);
    let p = r2.mul_add(p, c45);
    let p = r2.mul_add(p, c23);
    let p = r2.mul_add(p, c01);
    finish(x, r.mul_add(p, n))
}
