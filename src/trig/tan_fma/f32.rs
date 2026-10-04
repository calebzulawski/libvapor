/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/tanf.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2021-2025, Arm Limited.
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

const POLY_F32: [f32; 6] = [
    f32::from_bits(0x3eaaaaa8), // 0x1.55555p-2
    f32::from_bits(0x3e088b30), // 0x1.11166p-3
    f32::from_bits(0x3d5c453c), // 0x1.b88a78p-5
    f32::from_bits(0x3cbdabab), // 0x1.7b5756p-6
    f32::from_bits(0x3ba77a67), // 0x1.4ef4cep-8
    f32::from_bits(0x3c070f3a), // 0x1.0e1e74p-7
];

/// Computes tan(x) using fused multiply-add reduction and approximation.
#[inline]
pub fn tan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let slow = x.abs().simd_ge(Simd::splat(32768.0)) | !x.is_finite();
    let a = slow.select(Simd::splat(0.0), x);
    let shift = Simd::splat(12582912.0);
    let q = a.mul_add(Simd::splat(f32::from_bits(0x3f22f983)), shift);
    let n = q - shift;
    let odd = (q.to_bits() & Simd::splat(1)).simd_ne(Simd::splat(0));
    let r = n.mul_add(Simd::splat(-f32::from_bits(0x3fc90fdb)), a);
    let r = n.mul_add(Simd::splat(f32::from_bits(0x333bbd2e)), r);
    let r = n.mul_add(Simd::splat(f32::from_bits(0x26f72ced)), r);
    let z = odd.select(-r, r);
    let z2 = r * r;
    let z4 = z2 * z2;
    let z8 = z4 * z4;
    let c = POLY_F32.map(Simd::splat);
    let p01 = z2.mul_add(c[1], c[0]);
    let p23 = z2.mul_add(c[3], c[2]);
    let p45 = z2.mul_add(c[5], c[4]);
    let p = z8.mul_add(p45, z4.mul_add(p23, p01));
    let y = (z * z2).mul_add(p, z);
    let denominator = odd.select(y, Simd::splat(1.0));
    let y = odd.select(Simd::<f32, N>::splat(1.0) / denominator, y);
    let tiny = x.abs().simd_lt(Simd::splat(1.0e-5));
    let y = tiny.select(x, y);
    if !slow.any() {
        return y;
    }
    slow.select(
        super::super::tan::tan_f32(slow.select(x, Simd::splat(0.0))),
        y,
    )
}
