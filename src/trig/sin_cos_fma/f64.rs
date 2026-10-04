/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/sin.c and cos.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2025, Arm Limited.
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

use super::super::sin_cos_non_fma::full_range_sincos_f64 as sincos_f64;
use core::simd::prelude::*;
use std::simd::StdFloat;

const POLY_F64: [f64; 7] = [
    f64::from_bits(0xbfc555555555547b), // -0x1.555555555547bp-3
    f64::from_bits(0x3f81111111108a4d), //  0x1.1111111108a4dp-7
    f64::from_bits(0xbf2a01a019936f27), // -0x1.a01a019936f27p-13
    f64::from_bits(0x3ec71de37a97d93e), //  0x1.71de37a97d93ep-19
    f64::from_bits(0xbe5ae633919987c6), // -0x1.ae633919987c6p-26
    f64::from_bits(0x3de60e277ae07cec), //  0x1.60e277ae07cecp-33
    f64::from_bits(0xbd69e9540300a100), // -0x1.9e9540300a1p-41
];

#[inline]
pub(crate) fn sin_cos_f64<const N: usize, const COS: bool>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let slow = ax.simd_ge(Simd::splat(8388608.0)) | !x.is_finite();
    let a = slow.select(Simd::splat(0.0), ax);
    let inv_pi = Simd::splat(f64::from_bits(0x3fd45f306dc9c883));
    let count = if COS {
        a.mul_add(inv_pi, Simd::splat(0.5))
    } else {
        a * inv_pi
    };
    let shift = Simd::splat(f64::from_bits(0x4338000000000000));
    let shifted = count + shift;
    let odd = shifted.to_bits() << 63;
    let mut n = shifted - shift;
    if COS {
        n -= Simd::splat(0.5);
    }

    let r = n.mul_add(Simd::splat(-f64::from_bits(0x400921fb54442d18)), a);
    let r = n.mul_add(Simd::splat(-f64::from_bits(0x3ca1a62633145c06)), r);
    let r = n.mul_add(Simd::splat(-f64::from_bits(0x395c1cd129024e09)), r);
    let r2 = r * r;
    let r3 = r2 * r;
    let r4 = r2 * r2;
    let p45 = r2.mul_add(Simd::splat(POLY_F64[5]), Simd::splat(POLY_F64[4]));
    let p23 = r2.mul_add(Simd::splat(POLY_F64[3]), Simd::splat(POLY_F64[2]));
    let p01 = r2.mul_add(Simd::splat(POLY_F64[1]), Simd::splat(POLY_F64[0]));
    let p = r4.mul_add(Simd::splat(POLY_F64[6]), p45);
    let p = r4.mul_add(p, p23);
    let p = r4.mul_add(p, p01);
    let y = r3.mul_add(p, r);
    let sign = if COS {
        odd
    } else {
        odd ^ (x.to_bits() & Simd::splat(1 << 63))
    };
    let y = Simd::from_bits(y.to_bits() ^ sign);
    let tiny = ax.simd_lt(Simd::splat(f64::from_bits(0x3e40000000000000)));
    let y = tiny.select(if COS { Simd::splat(1.0) } else { x }, y);
    if !slow.any() {
        return y;
    }
    let (s, c) = sincos_f64(slow.select(x, Simd::splat(0.0)));
    slow.select(if COS { c } else { s }, y)
}
