/*
 * Derived from musl src/math/exp2f_data.c, with changes.
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

pub(crate) const TABLE_BITS_F32: u64 = 5;

pub(crate) const TABLE_F32: [u64; 32] = [
    0x3ff0000000000000,
    0x3fefd9b0d3158574,
    0x3fefb5586cf9890f,
    0x3fef9301d0125b51,
    0x3fef72b83c7d517b,
    0x3fef54873168b9aa,
    0x3fef387a6e756238,
    0x3fef1e9df51fdee1,
    0x3fef06fe0a31b715,
    0x3feef1a7373aa9cb,
    0x3feedea64c123422,
    0x3feece086061892d,
    0x3feebfdad5362a27,
    0x3feeb42b569d4f82,
    0x3feeab07dd485429,
    0x3feea47eb03a5585,
    0x3feea09e667f3bcd,
    0x3fee9f75e8ec5f74,
    0x3feea11473eb0187,
    0x3feea589994cce13,
    0x3feeace5422aa0db,
    0x3feeb737b0cdc5e5,
    0x3feec49182a3f090,
    0x3feed503b23e255d,
    0x3feee89f995ad3ad,
    0x3feeff76f2fb5e47,
    0x3fef199bdd85529c,
    0x3fef3720dcef9069,
    0x3fef5818dcfba487,
    0x3fef7c97337b9b5f,
    0x3fefa4afa2a490da,
    0x3fefd0765b6e4540,
];

pub(crate) const EXP2_POLY_F32: [f64; 3] = [
    f64::from_bits(0x3fac6af84b912394), // 0x1.c6af84b912394p-5
    f64::from_bits(0x3fcebfce50fac4f3), // 0x1.ebfce50fac4f3p-3
    f64::from_bits(0x3fe62e42ff0c52d6), // 0x1.62e42ff0c52d6p-1
];

pub(crate) const EXP_POLY_F32: [f64; 3] = [
    EXP2_POLY_F32[0] / (32.0 * 32.0 * 32.0),
    EXP2_POLY_F32[1] / (32.0 * 32.0),
    EXP2_POLY_F32[2] / 32.0,
];

pub(crate) const SHIFT: f64 = f64::from_bits(0x4338000000000000);

// 0x1.8p+52
pub(crate) const EXP2_SHIFT_F32: f64 = SHIFT / 32.0;

pub(crate) const INV_LN2_SCALED_F32: f64 = f64::from_bits(0x3ff71547652b82fe) * 32.0;
