/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/tan.c; pi split: sin.c and cos.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2025, Arm Limited.
 * Copyright (c) 2023-2025, Arm Limited.
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

const POLY_F64: [f64; 9] = [
    f64::from_bits(0x3fd5555555555556), // 0x1.5555555555556p-2
    f64::from_bits(0x3fc1111111110a63), // 0x1.1111111110a63p-3
    f64::from_bits(0x3faba1ba1bb46414), // 0x1.ba1ba1bb46414p-5
    f64::from_bits(0x3f9664f47e5b5445), // 0x1.664f47e5b5445p-6
    f64::from_bits(0x3f8226e5e5ecdfa3), // 0x1.226e5e5ecdfa3p-7
    f64::from_bits(0x3f6d6c7ddbf87047), // 0x1.d6c7ddbf87047p-9
    f64::from_bits(0x3f57ea75d05b583e), // 0x1.7ea75d05b583ep-10
    f64::from_bits(0x3f4289f22964a03c), // 0x1.289f22964a03cp-11
    f64::from_bits(0x3f34e4fd14147622), // 0x1.4e4fd14147622p-12
];

/// Computes tan(x) using fused reduction and the tangent double-angle formula.
#[inline]
pub fn tan_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let slow = x.abs().simd_ge(Simd::splat(8388608.0)) | !x.is_finite();
    let a = slow.select(Simd::splat(0.0), x);
    let shift = Simd::splat(6755399441055744.0);
    let q = a.mul_add(Simd::splat(f64::from_bits(0x3fe45f306dc9c883)), shift);
    let n = q - shift;
    let odd = (q.to_bits() & Simd::splat(1)).simd_ne(Simd::splat(0));
    let r = n.mul_add(Simd::splat(-f64::from_bits(0x3ff921fb54442d18)), a);
    let r = n.mul_add(Simd::splat(-f64::from_bits(0x3c91a62633145c06)), r);
    let r = n.mul_add(Simd::splat(-f64::from_bits(0x394c1cd129024e09)), r);
    let r = r * Simd::splat(0.5);
    let r2 = r * r;
    let r4 = r2 * r2;
    let r8 = r4 * r4;
    let c = POLY_F64.map(Simd::splat);
    let p12 = r2.mul_add(c[2], c[1]);
    let p34 = r2.mul_add(c[4], c[3]);
    let p56 = r2.mul_add(c[6], c[5]);
    let p78 = r2.mul_add(c[8], c[7]);
    let p = r8.mul_add(r4.mul_add(p78, p56), r4.mul_add(p34, p12));
    let p = r2.mul_add(p, c[0]);
    let p = r2.mul_add(p * r, r);
    let numerator = p.mul_add(p, Simd::splat(-1.0));
    let denominator = p + p;
    let y = odd.select(numerator, -denominator) / odd.select(denominator, numerator);
    let tiny = x.abs().simd_lt(Simd::splat(1.0e-9));
    let y = tiny.select(x, y);
    if !slow.any() {
        return y;
    }
    slow.select(
        super::super::tan::tan_f64(slow.select(x, Simd::splat(0.0))),
        y,
    )
}
