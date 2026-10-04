/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/v_sincosf_common.h, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
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

const SIN_F32: [f32; 3] = [
    f32::from_bits(0xbe2aaaa3), // -0x1.555546p-3
    f32::from_bits(0x3c0883b0), // 0x1.11076p-7
    f32::from_bits(0xb94ca75a), // -0x1.994eb4p-13
];

const COS_F32: [f32; 3] = [
    f32::from_bits(0x3d2aaaa5), // 0x1.55554ap-5
    f32::from_bits(0xbab6060d), // -0x1.6c0c1ap-10
    f32::from_bits(0x37ccf077), // 0x1.99e0eep-16
];

const PIO2_F32: [f32; 3] = [
    f32::from_bits(0x3fc90fdb), // 0x1.921fb6p+0
    f32::from_bits(0xb33bbd2e), // -0x1.777a5cp-25
    f32::from_bits(0xa6f72ced), // -0x1.ee59dap-50
];

#[inline]
fn sin_poly_f32<const N: usize>(z: Simd<f32, N>) -> Simd<f32, N> {
    let c = SIN_F32.map(Simd::splat);
    z.mul_add(z.mul_add(c[2], c[1]), c[0])
}

#[inline]
fn cos_poly_f32<const N: usize>(z: Simd<f32, N>) -> Simd<f32, N> {
    let c = COS_F32.map(Simd::splat);
    z.mul_add(z.mul_add(c[2], c[1]), c[0])
}

/// Computes both sine and cosine with a shared FMA range reduction.
#[inline]
pub(crate) fn sincos_f32<const N: usize>(x: Simd<f32, N>) -> (Simd<f32, N>, Simd<f32, N>) {
    let ax = x.abs();
    let slow = ax.simd_ge(Simd::splat(1048576.0f32)) | !x.is_finite();
    let a = slow.select(Simd::splat(0.0), ax);
    let shift = Simd::splat(12582912.0f32);
    let q = a.mul_add(Simd::splat(f32::from_bits(0x3f22f983)), shift);
    let n = q - shift;
    let quadrant = q.to_bits() & Simd::splat(3);
    let r = n.mul_add(Simd::splat(-PIO2_F32[0]), a);
    let r = n.mul_add(Simd::splat(-PIO2_F32[1]), r);
    let r = n.mul_add(Simd::splat(-PIO2_F32[2]), r);
    let r2 = r * r;
    let s = (r * r2).mul_add(sin_poly_f32(r2), r);
    let p = r2.mul_add(cos_poly_f32(r2), Simd::splat(-0.5));
    let c = r2.mul_add(p, Simd::splat(1.0));
    let swap = (quadrant & Simd::splat(1)).simd_ne(Simd::splat(0));
    let ss = swap.select(c, s);
    let cc = swap.select(s, c);
    let sin_sign = ((quadrant & Simd::splat(2)) << (31 - 1)) ^ (x.to_bits() & Simd::splat(1 << 31));
    let cos_sign = ((quadrant + Simd::splat(1)) & Simd::splat(2)) << (31 - 1);
    let ss = Simd::from_bits(ss.to_bits() ^ sin_sign);
    let cc = Simd::from_bits(cc.to_bits() ^ cos_sign);
    let tiny = ax.simd_lt(Simd::splat(1.0e-4f32));
    let ss = tiny.select(x, ss);
    let cc = tiny.select(Simd::splat(1.0), cc);
    if !slow.any() {
        return (ss, cc);
    }
    let (fs, fc) =
        super::super::sin_cos_non_fma::full_range_sincos_f32(slow.select(x, Simd::splat(0.0)));
    (slow.select(fs, ss), slow.select(fc, cc))
}
