/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/v_sincos_common.h; pi split: sin.c and cos.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2025, Arm Limited.
 * Copyright (c) 2023-2024, Arm Limited.
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

const SIN_F64: [f64; 7] = [
    f64::from_bits(0xbfc555555555547b), // -0x1.555555555547bp-3
    f64::from_bits(0x3f81111111108a4d), // 0x1.1111111108a4dp-7
    f64::from_bits(0xbf2a01a019936f27), // -0x1.a01a019936f27p-13
    f64::from_bits(0x3ec71de37a97d93e), // 0x1.71de37a97d93ep-19
    f64::from_bits(0xbe5ae633919987c6), // -0x1.ae633919987c6p-26
    f64::from_bits(0x3de60e277ae07cec), // 0x1.60e277ae07cecp-33
    f64::from_bits(0xbd69e9540300a100), // -0x1.9e9540300a1p-41
];

const COS_F64: [f64; 6] = [
    f64::from_bits(0x3fa555555555554c), // 0x1.555555555554cp-5
    f64::from_bits(0xbf56c16c16c1521f), // -0x1.6c16c16c1521fp-10
    f64::from_bits(0x3efa01a019cbf62a), // 0x1.a01a019cbf62ap-16
    f64::from_bits(0xbe927e4f812b681e), // -0x1.27e4f812b681ep-22
    f64::from_bits(0x3e21ee9f152a57cd), // 0x1.1ee9f152a57cdp-29
    f64::from_bits(0xbda8fb131098404b), // -0x1.8fb131098404bp-37
];

const PIO2_F64: [f64; 3] = [
    f64::from_bits(0x3ff921fb54442d18),
    f64::from_bits(0x3c91a62633145c06),
    f64::from_bits(0x394c1cd129024e09),
];

#[inline]
fn sin_poly_f64<const N: usize>(z: Simd<f64, N>) -> Simd<f64, N> {
    let c = SIN_F64.map(Simd::splat);
    let z2 = z * z;
    let p0 = z.mul_add(c[1], c[0]);
    let p1 = z.mul_add(c[3], c[2]);
    let p2 = z.mul_add(c[5], c[4]);
    let p = z2.mul_add(c[6], p2);
    let p = z2.mul_add(p, p1);
    let p = z2.mul_add(p, p0);
    p
}

#[inline]
fn cos_poly_f64<const N: usize>(z: Simd<f64, N>) -> Simd<f64, N> {
    let c = COS_F64.map(Simd::splat);
    let z2 = z * z;
    let p0 = z.mul_add(c[1], c[0]);
    let p1 = z.mul_add(c[3], c[2]);
    let p2 = z.mul_add(c[5], c[4]);
    let p = p2;
    let p = z2.mul_add(p, p1);
    let p = z2.mul_add(p, p0);
    p
}

/// Computes both sine and cosine with a shared FMA range reduction.
#[inline]
pub(crate) fn sincos_f64<const N: usize>(x: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    let ax = x.abs();
    let slow = ax.simd_ge(Simd::splat(8388608.0f64)) | !x.is_finite();
    let a = slow.select(Simd::splat(0.0), ax);
    let shift = Simd::splat(6755399441055744.0f64);
    let q = a.mul_add(Simd::splat(f64::from_bits(0x3fe45f306dc9c882)), shift);
    let n = q - shift;
    let quadrant = q.to_bits() & Simd::splat(3);
    let r = n.mul_add(Simd::splat(-PIO2_F64[0]), a);
    let r = n.mul_add(Simd::splat(-PIO2_F64[1]), r);
    let r = n.mul_add(Simd::splat(-PIO2_F64[2]), r);
    let r2 = r * r;
    let s = (r * r2).mul_add(sin_poly_f64(r2), r);
    let p = r2.mul_add(cos_poly_f64(r2), Simd::splat(-0.5));
    let c = r2.mul_add(p, Simd::splat(1.0));
    let swap = (quadrant & Simd::splat(1)).simd_ne(Simd::splat(0));
    let ss = swap.select(c, s);
    let cc = swap.select(s, c);
    let sin_sign = ((quadrant & Simd::splat(2)) << (63 - 1)) ^ (x.to_bits() & Simd::splat(1 << 63));
    let cos_sign = ((quadrant + Simd::splat(1)) & Simd::splat(2)) << (63 - 1);
    let ss = Simd::from_bits(ss.to_bits() ^ sin_sign);
    let cc = Simd::from_bits(cc.to_bits() ^ cos_sign);
    let tiny = ax.simd_lt(Simd::splat(1.0e-9f64));
    let ss = tiny.select(x, ss);
    let cc = tiny.select(Simd::splat(1.0), cc);
    if !slow.any() {
        return (ss, cc);
    }
    let (fs, fc) =
        super::super::sin_cos_non_fma::full_range_sincos_f64(slow.select(x, Simd::splat(0.0)));
    (slow.select(fs, ss), slow.select(fc, cc))
}
