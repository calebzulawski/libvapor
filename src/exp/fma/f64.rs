/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/exp.c and exp2.c; table: math/aarch64/v_exp_data.c, with changes.
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

const EXP_POLY: [f64; 3] = [
    f64::from_bits(0x3fdffffffffffd43), // 0x1.ffffffffffd43p-2
    f64::from_bits(0x3fc55555c75adbb2), // 0x1.55555c75adbb2p-3
    f64::from_bits(0x3fa55555da646206), // 0x1.55555da646206p-5
];

const EXP2_POLY: [f64; 4] = [
    f64::from_bits(0x3fe62e42fefa3686), // 0x1.62e42fefa3686p-1
    f64::from_bits(0x3fcebfbdff82c241), // 0x1.ebfbdff82c241p-3
    f64::from_bits(0x3fac6b09b16de99a), // 0x1.c6b09b16de99ap-5
    f64::from_bits(0x3f83b2abf5571ad8), // 0x1.3b2abf5571ad8p-7
];

const SCALE: [u64; 128] = [
    0x3ff0000000000000,
    0x3feff63da9fb3335,
    0x3fefec9a3e778061,
    0x3fefe315e86e7f85,
    0x3fefd9b0d3158574,
    0x3fefd06b29ddf6de,
    0x3fefc74518759bc8,
    0x3fefbe3ecac6f383,
    0x3fefb5586cf9890f,
    0x3fefac922b7247f7,
    0x3fefa3ec32d3d1a2,
    0x3fef9b66affed31b,
    0x3fef9301d0125b51,
    0x3fef8abdc06c31cc,
    0x3fef829aaea92de0,
    0x3fef7a98c8a58e51,
    0x3fef72b83c7d517b,
    0x3fef6af9388c8dea,
    0x3fef635beb6fcb75,
    0x3fef5be084045cd4,
    0x3fef54873168b9aa,
    0x3fef4d5022fcd91d,
    0x3fef463b88628cd6,
    0x3fef3f49917ddc96,
    0x3fef387a6e756238,
    0x3fef31ce4fb2a63f,
    0x3fef2b4565e27cdd,
    0x3fef24dfe1f56381,
    0x3fef1e9df51fdee1,
    0x3fef187fd0dad990,
    0x3fef1285a6e4030b,
    0x3fef0cafa93e2f56,
    0x3fef06fe0a31b715,
    0x3fef0170fc4cd831,
    0x3feefc08b26416ff,
    0x3feef6c55f929ff1,
    0x3feef1a7373aa9cb,
    0x3feeecae6d05d866,
    0x3feee7db34e59ff7,
    0x3feee32dc313a8e5,
    0x3feedea64c123422,
    0x3feeda4504ac801c,
    0x3feed60a21f72e2a,
    0x3feed1f5d950a897,
    0x3feece086061892d,
    0x3feeca41ed1d0057,
    0x3feec6a2b5c13cd0,
    0x3feec32af0d7d3de,
    0x3feebfdad5362a27,
    0x3feebcb299fddd0d,
    0x3feeb9b2769d2ca7,
    0x3feeb6daa2cf6642,
    0x3feeb42b569d4f82,
    0x3feeb1a4ca5d920f,
    0x3feeaf4736b527da,
    0x3feead12d497c7fd,
    0x3feeab07dd485429,
    0x3feea9268a5946b7,
    0x3feea76f15ad2148,
    0x3feea5e1b976dc09,
    0x3feea47eb03a5585,
    0x3feea34634ccc320,
    0x3feea23882552225,
    0x3feea155d44ca973,
    0x3feea09e667f3bcd,
    0x3feea012750bdabf,
    0x3fee9fb23c651a2f,
    0x3fee9f7df9519484,
    0x3fee9f75e8ec5f74,
    0x3fee9f9a48a58174,
    0x3fee9feb564267c9,
    0x3feea0694fde5d3f,
    0x3feea11473eb0187,
    0x3feea1ed0130c132,
    0x3feea2f336cf4e62,
    0x3feea427543e1a12,
    0x3feea589994cce13,
    0x3feea71a4623c7ad,
    0x3feea8d99b4492ed,
    0x3feeaac7d98a6699,
    0x3feeace5422aa0db,
    0x3feeaf3216b5448c,
    0x3feeb1ae99157736,
    0x3feeb45b0b91ffc6,
    0x3feeb737b0cdc5e5,
    0x3feeba44cbc8520f,
    0x3feebd829fde4e50,
    0x3feec0f170ca07ba,
    0x3feec49182a3f090,
    0x3feec86319e32323,
    0x3feecc667b5de565,
    0x3feed09bec4a2d33,
    0x3feed503b23e255d,
    0x3feed99e1330b358,
    0x3feede6b5579fdbf,
    0x3feee36bbfd3f37a,
    0x3feee89f995ad3ad,
    0x3feeee07298db666,
    0x3feef3a2b84f15fb,
    0x3feef9728de5593a,
    0x3feeff76f2fb5e47,
    0x3fef05b030a1064a,
    0x3fef0c1e904bc1d2,
    0x3fef12c25bd71e09,
    0x3fef199bdd85529c,
    0x3fef20ab5fffd07a,
    0x3fef27f12e57d14b,
    0x3fef2f6d9406e7b5,
    0x3fef3720dcef9069,
    0x3fef3f0b555dc3fa,
    0x3fef472d4a07897c,
    0x3fef4f87080d89f2,
    0x3fef5818dcfba487,
    0x3fef60e316c98398,
    0x3fef69e603db3285,
    0x3fef7321f301b460,
    0x3fef7c97337b9b5f,
    0x3fef864614f5a129,
    0x3fef902ee78b3ff6,
    0x3fef9a51fbc74c83,
    0x3fefa4afa2a490da,
    0x3fefaf482d8e67f1,
    0x3fefba1bee615a27,
    0x3fefc52b376bba97,
    0x3fefd0765b6e4540,
    0x3fefdbfdad9cbe14,
    0x3fefe7c1819e90d8,
    0x3feff3c22b8f71f1,
];

#[inline]
fn exp_f64_kernel<const N: usize, const BASE2: bool>(x: Simd<f64, N>) -> Simd<f64, N> {
    let limit = if BASE2 { 1022.0 } else { 704.0 };
    if !x.abs().simd_lt(Simd::splat(limit)).all() {
        return special_f64::<N, BASE2>(x);
    }
    let a = x;
    let shift = Simd::splat(if BASE2 {
        52776558133248.0
    } else {
        6755399441055744.0
    });
    let shifted = if BASE2 {
        a + shift
    } else {
        a.mul_add(Simd::splat(f64::from_bits(0x40671547652b82fe)), shift)
    };
    let n = shifted - shift;
    let r = if BASE2 {
        a - n
    } else {
        let r = n.mul_add(Simd::splat(-f64::from_bits(0x3f762e42fefa39ef)), a);
        n.mul_add(Simd::splat(-f64::from_bits(0x3c0abc9e3b39803f)), r)
    };
    let r2 = r * r;
    let poly = if BASE2 {
        let c = EXP2_POLY.map(Simd::splat);
        let p01 = r.mul_add(c[1], c[0]);
        let p23 = r.mul_add(c[3], c[2]);
        r * r2.mul_add(p23, p01)
    } else {
        let c = EXP_POLY.map(Simd::splat);
        let p = r.mul_add(c[1], c[0]);
        let p = r2.mul_add(c[2], p);
        r2.mul_add(p, r)
    };
    let u = shifted.to_bits();
    let index = (u & Simd::splat(127)).cast::<usize>();
    let table = Simd::gather_or(&SCALE, index, Simd::splat(0));
    let scale = Simd::<f64, N>::from_bits(table + (u << 45));
    let y = scale.mul_add(poly, scale);
    y
}

/// Computes e^x using FMA reduction and a single scale-table lookup.
#[inline]
pub fn exp_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    exp_f64_kernel::<N, false>(x)
}

/// Computes 2^x using FMA approximation and a single scale-table lookup.
#[inline]
pub fn exp2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    exp_f64_kernel::<N, true>(x)
}

#[cold]
#[inline(never)]
fn special_f64<const N: usize, const BASE2: bool>(x: Simd<f64, N>) -> Simd<f64, N> {
    let limit = if BASE2 { 1022.0 } else { 704.0 };
    let slow = !x.abs().simd_lt(Simd::splat(limit));
    let a = slow.select(Simd::splat(0.0), x);
    let y = exp_f64_kernel::<N, BASE2>(a);
    let a = slow.select(x, Simd::splat(0.0));
    let fallback = if BASE2 {
        super::super::full_range::exp2_f64(a)
    } else {
        super::super::full_range::exp_f64(a)
    };
    slow.select(fallback, y)
}
