/*
 * Derived from musl src/math/expf.c and exp2f.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2017-2018, Arm Limited.
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

use super::super::data::*;
use core::simd::prelude::*;

fn exp_poly<const N: usize>(ki: Simd<u64, N>, r: Simd<f64, N>, poly: [f64; 3]) -> Simd<f32, N> {
    // TABLE_F32[i] stores the bits of 2^(i/32) minus (i << 47). Adding the
    // shifted rounding bits reconstructs 2^(k/32), including negative k.
    let i = ki & Simd::splat(31);
    let i = i.cast::<usize>();
    let t = Simd::<u64, N>::gather_or(&TABLE_F32, i, Simd::splat(0));
    let s = Simd::<f64, N>::from_bits(t + (ki << (52 - TABLE_BITS_F32)));

    let z = Simd::splat(poly[0]) * r + Simd::splat(poly[1]);
    let r2 = r * r;
    let y = Simd::splat(poly[2]) * r + Simd::splat(1.0);
    let y = z * r2 + y;
    let y = y * s;
    y.cast::<f32>()
}

/// Computes 2^x for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn exp2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let overflow = x.simd_ge(Simd::splat(128.0));
    let underflow = x.simd_le(Simd::splat(-150.0));
    let nan = x.is_nan();

    // Selects evaluate both branches. Keep exceptional lanes out
    // of the reduction and restore their results afterward.
    let xd = (overflow | underflow | nan)
        .select(Simd::splat(0.0), x)
        .cast::<f64>();
    let kd = xd + Simd::splat(EXP2_SHIFT_F32);
    let ki = kd.to_bits();
    let kd = kd - Simd::splat(EXP2_SHIFT_F32);
    let r = xd - kd;
    let y = exp_poly(ki, r, EXP2_POLY_F32);

    nan.select(
        x + x,
        overflow.select(
            Simd::splat(f32::INFINITY),
            underflow.select(Simd::splat(0.0), y),
        ),
    )
}

/// Computes e^x for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn exp_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let overflow = x.simd_gt(Simd::splat(f32::from_bits(0x42b17217)));
    let underflow = x.simd_lt(Simd::splat(f32::from_bits(0xc2cff1b4)));
    let nan = x.is_nan();
    let xd = (overflow | underflow | nan)
        .select(Simd::splat(0.0), x)
        .cast::<f64>();

    // x*32/ln(2) = k + r, with |r| <= 1/2.
    let z = Simd::splat(INV_LN2_SCALED_F32) * xd;
    let kd = z + Simd::splat(SHIFT);
    let ki = kd.to_bits();
    let kd = kd - Simd::splat(SHIFT);
    let r = z - kd;
    let y = exp_poly(ki, r, EXP_POLY_F32);

    nan.select(
        x + x,
        overflow.select(
            Simd::splat(f32::INFINITY),
            underflow.select(Simd::splat(0.0), y),
        ),
    )
}
