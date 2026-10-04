/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log1pf.c and v_log1pf_inline.h, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2022-2025, Arm Limited.
 * Copyright (c) 2022-2024, Arm Limited.
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

trait UnfusedMulAdd {
    fn mul_add(self, b: Self, c: Self) -> Self;
}

impl<T, const N: usize> UnfusedMulAdd for Simd<T, N>
where
    T: core::simd::SimdElement,
    Simd<T, N>: core::ops::Mul<Output = Self> + core::ops::Add<Output = Self>,
{
    #[inline]
    fn mul_add(self, b: Self, c: Self) -> Self {
        self * b + c
    }
}

const COEFF_F32: [f32; 8] = [
    f32::from_bits(0x3eaaaad5), // 0x1.5555aap-2
    f32::from_bits(0xbe80001c), // -0x1.000038p-2
    f32::from_bits(0x3e4cb3ae), // 0x1.99675cp-3
    f32::from_bits(0xbe2a77bc), // -0x1.54ef78p-3
    f32::from_bits(0x3e1450fa), // 0x1.28a1f4p-3
    f32::from_bits(0xbe06d488), // -0x1.0da91p-3
    f32::from_bits(0x3dd5e5b0), // 0x1.abcb6p-4
    f32::from_bits(0xbd3786af), // -0x1.6f0d5ep-5
];

#[inline]
fn poly_f32<const N: usize>(m: Simd<f32, N>) -> Simd<f32, N> {
    let c = COEFF_F32.map(Simd::splat);
    let m2 = m * m;
    let q = m.mul_add(c[0], Simd::splat(-0.5));
    let p67 = m.mul_add(c[7], c[6]);
    let p45 = m.mul_add(c[5], c[4]);
    let p23 = m.mul_add(c[3], c[2]);
    let p = m2.mul_add(p67, p45);
    let p = m2.mul_add(p, p23);
    let p = m.mul_add(p, c[1]);
    let p = m2 * p;
    let p = m2.mul_add(p, m);
    m2.mul_add(q, p)
}

/// Computes log(1+x) accurately near zero, using non-FMA polynomial reduction.
#[inline]
pub fn log1p_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let special = !x.is_finite() | x.simd_le(Simd::splat(-1.0));
    let a = special.select(Simd::splat(0.0), x);
    let u = a + Simd::splat(1.0);
    let k = (u.to_bits() - Simd::splat(0x3f400000)) & Simd::splat(0xff800000);
    let s = Simd::<f32, N>::from_bits(Simd::splat(0x40800000) - k);
    let m = Simd::<f32, N>::from_bits(a.to_bits() - k);
    let m = m + s.mul_add(Simd::splat(0.25), Simd::splat(-1.0));
    let p = poly_f32(m);
    let exponent = k.cast::<i32>().cast::<f32>() * Simd::splat(f32::from_bits(0x34000000));
    let y = exponent.mul_add(Simd::splat(f32::from_bits(0x3f317218)), p);
    let y = x
        .abs()
        .simd_lt(Simd::splat(f32::from_bits(0x33000000)))
        .select(x, y);
    if !special.any() {
        return y;
    }
    let exceptional = x
        .simd_eq(Simd::splat(-1.0))
        .select(Simd::splat(f32::NEG_INFINITY), Simd::splat(f32::NAN));
    let exceptional = x.simd_eq(Simd::splat(f32::INFINITY)).select(x, exceptional);
    let exceptional = x.is_nan().select(x + x, exceptional);
    special.select(exceptional, y)
}
