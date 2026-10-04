/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/expf.c and exp2f.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2026, Arm Limited.
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

const EXPF_POLY: [f32; 5] = [
    f32::from_bits(0x3c072010), // 0x1.0e4020p-7
    f32::from_bits(0x3d2b9f17), // 0x1.573e2ep-5
    f32::from_bits(0x3e2aaf33), // 0x1.555e66p-3
    f32::from_bits(0x3efffedb), // 0x1.fffdb6p-2
    f32::from_bits(0x3f7ffff6), // 0x1.ffffecp-1
];

const EXP2F_POLY: [f32; 5] = [
    f32::from_bits(0x3aaccbbd), // 0x1.59977ap-10
    f32::from_bits(0x3c1e74f2), // 0x1.3ce9e4p-7
    f32::from_bits(0x3d635e99), // 0x1.c6bd32p-5
    f32::from_bits(0x3e75fcde), // 0x1.ebf9bcp-3
    f32::from_bits(0x3f317211), // 0x1.62e422p-1
];

#[inline]
fn exp_f32_kernel<const N: usize, const BASE2: bool>(x: Simd<f32, N>) -> Simd<f32, N> {
    let limit = if BASE2 { 126.5 } else { 87.0 };
    if !x.abs().simd_lt(Simd::splat(limit)).all() {
        return special_f32::<N, BASE2>(x);
    }
    let a = x;
    let z = if BASE2 {
        a
    } else {
        a * Simd::splat(f32::from_bits(0x3fb8aa3b))
    };
    let shift = Simd::splat(12582912.0f32);
    let shifted = z + shift;
    let n = shifted - shift;
    let r = if BASE2 {
        a - n
    } else {
        let r = n.mul_add(Simd::splat(-f32::from_bits(0x3f317200)), a);
        n.mul_add(Simd::splat(-f32::from_bits(0x35bfbe8e)), r)
    };
    let c = if BASE2 { EXP2F_POLY } else { EXPF_POLY };
    let c = c.map(Simd::splat);
    let r2 = r * r;
    let p = r.mul_add(c[0], c[1]);
    let q = r.mul_add(c[2], c[3]);
    let q = r2.mul_add(p, q);
    let poly = r2.mul_add(q, c[4] * r);
    let scale = Simd::<f32, N>::from_bits((shifted.to_bits() << 23) + Simd::splat(0x3f800000));
    let y = scale.mul_add(poly, scale);
    y
}

/// Computes e^x using a table-free FMA polynomial.
#[inline]
pub fn exp_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    exp_f32_kernel::<N, false>(x)
}

/// Computes 2^x using a table-free FMA polynomial.
#[inline]
pub fn exp2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    exp_f32_kernel::<N, true>(x)
}

#[cold]
#[inline(never)]
fn special_f32<const N: usize, const BASE2: bool>(x: Simd<f32, N>) -> Simd<f32, N> {
    let limit = if BASE2 { 126.5 } else { 87.0 };
    let slow = !x.abs().simd_lt(Simd::splat(limit));
    let a = slow.select(Simd::splat(0.0), x);
    let y = exp_f32_kernel::<N, BASE2>(a);
    let a = slow.select(x, Simd::splat(0.0));
    let fallback = if BASE2 {
        super::super::full_range::exp2_f32(a)
    } else {
        super::super::full_range::exp_f32(a)
    };
    slow.select(fallback, y)
}
