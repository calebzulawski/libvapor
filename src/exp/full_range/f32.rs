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
use simd_macros::vectorize;

fn exp_poly<const N: usize>(ki: Simd<u64, N>, r: Simd<f64, N>, poly: [f64; 3]) -> Simd<f32, N> {
    vectorize!(N, {
        // TABLE_F32[i] stores the bits of 2^(i/32) minus (i << 47). Adding the
        // shifted rounding bits reconstructs 2^(k/32), including negative k.
        let i = ki & 31;
        let i = i as usize;
        let t = <u64>::gather_or(&TABLE_F32, i, 0);
        let s = <f64>::from_bits(t + (ki << scalar!(52 - TABLE_BITS_F32)));

        let z = scalar!(poly[0]) * r + scalar!(poly[1]);
        let r2 = r * r;
        let y = scalar!(poly[2]) * r + 1.0;
        let y = z * r2 + y;
        let y = y * s;
        y as f32
    })
}

/// Computes 2^x for each lane, assuming round-to-nearest, ties-to-even.
#[allow(
    unused_braces,
    unused_parens,
    reason = "vectorize! retains scalar branch braces and grouping"
)]
#[inline]
pub fn exp2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let overflow = x >= 128.0;
        let underflow = x <= -150.0;
        let nan = x.is_nan();

        // Selects evaluate both branches. Keep exceptional lanes out
        // of the reduction and restore their results afterward.
        let xd = (if overflow | underflow | nan { 0.0 } else { x }) as f64;
        let kd = xd + scalar!(EXP2_SHIFT_F32);
        let ki = kd.to_bits();
        let kd = kd - scalar!(EXP2_SHIFT_F32);
        let r = xd - kd;
        let y = exp_poly(ki, r, verbatim!(EXP2_POLY_F32));

        if nan {
            x + x
        } else if overflow {
            scalar!(f32::INFINITY)
        } else if underflow {
            0.0
        } else {
            y
        }
    })
}

/// Computes e^x for each lane, assuming round-to-nearest, ties-to-even.
#[allow(
    unused_braces,
    unused_parens,
    reason = "vectorize! retains scalar branch braces and grouping"
)]
#[inline]
pub fn exp_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let overflow = x > scalar!(f32::from_bits(0x42b17217));
        let underflow = x < scalar!(f32::from_bits(0xc2cff1b4));
        let nan = x.is_nan();
        let xd = (if overflow | underflow | nan { 0.0 } else { x }) as f64;

        // x*32/ln(2) = k + r, with |r| <= 1/2.
        let z = scalar!(INV_LN2_SCALED_F32) * xd;
        let kd = z + scalar!(SHIFT);
        let ki = kd.to_bits();
        let kd = kd - scalar!(SHIFT);
        let r = z - kd;
        let y = exp_poly(ki, r, verbatim!(EXP_POLY_F32));

        if nan {
            x + x
        } else if overflow {
            scalar!(f32::INFINITY)
        } else if underflow {
            0.0
        } else {
            y
        }
    })
}
