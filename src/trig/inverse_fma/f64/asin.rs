/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/asin.c and acos.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
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

#[inline]
fn asin_poly_f64<const N: usize>(z: Simd<f64, N>) -> Simd<f64, N> {
    let c0 = Simd::splat(f64::from_bits(0x3fc555555555554e)); // 0x1.555555555554ep-3
    let c1 = Simd::splat(f64::from_bits(0x3fb3333333337233)); // 0x1.3333333337233p-4
    let c2 = Simd::splat(f64::from_bits(0x3fa6db6db67f6d9f)); // 0x1.6db6db67f6d9fp-5
    let c3 = Simd::splat(f64::from_bits(0x3f9f1c71fbd29fbb)); // 0x1.f1c71fbd29fbbp-6
    let c4 = Simd::splat(f64::from_bits(0x3f96e8b264d467d6)); // 0x1.6e8b264d467d6p-6
    let c5 = Simd::splat(f64::from_bits(0x3f91c5997c357e9d)); // 0x1.1c5997c357e9dp-6
    let c6 = Simd::splat(f64::from_bits(0x3f8c86a22cd9389d)); // 0x1.c86a22cd9389dp-7
    let c7 = Simd::splat(f64::from_bits(0x3f8856073c22ebbe)); // 0x1.856073c22ebbep-7
    let c8 = Simd::splat(f64::from_bits(0x3f7fd1151acb6bed)); // 0x1.fd1151acb6bedp-8
    let c9 = Simd::splat(f64::from_bits(0x3f9087182f799c1d)); // 0x1.087182f799c1dp-6
    let c10 = Simd::splat(f64::from_bits(0xbf86602748120927)); // -0x1.6602748120927p-7
    let c11 = Simd::splat(f64::from_bits(0x3f9cfa0dd1f94780)); // 0x1.cfa0dd1f9478p-6
    let p0_0 = z.mul_add(c1, c0);
    let p0_1 = z.mul_add(c3, c2);
    let p0_2 = z.mul_add(c5, c4);
    let p0_3 = z.mul_add(c7, c6);
    let p0_4 = z.mul_add(c9, c8);
    let p0_5 = z.mul_add(c11, c10);
    let z2 = z * z;
    let p1_0 = z2.mul_add(p0_1, p0_0);
    let p1_1 = z2.mul_add(p0_3, p0_2);
    let p1_2 = z2.mul_add(p0_5, p0_4);
    let z4 = z2 * z2;
    let p2_0 = z4.mul_add(p1_1, p1_0);
    let z8 = z4 * z4;
    let p3_0 = z8.mul_add(p1_2, p2_0);
    p3_0
}

/// Computes asin(x), assuming round-to-nearest, ties-to-even.
#[inline]
pub fn asin_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let small = ax.simd_lt(Simd::<f64, N>::splat(0.5));
    let z2 = small.select(
        x * x,
        (-ax).mul_add(Simd::<f64, N>::splat(0.5), Simd::<f64, N>::splat(0.5)),
    );
    let z = small.select(ax, crate::sqrt_f64(z2));
    let p = (z * z2).mul_add(asin_poly_f64(z2), z);
    small
        .select(
            p,
            p.mul_add(
                Simd::<f64, N>::splat(-2.0),
                Simd::<f64, N>::splat(core::f64::consts::FRAC_PI_2),
            ),
        )
        .copysign(x)
}

/// Computes acos(x), assuming round-to-nearest, ties-to-even.
#[inline]
pub fn acos_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let ax = x.abs();
    let small = ax.simd_le(Simd::<f64, N>::splat(0.5));
    let z2 = small.select(
        x * x,
        (-ax).mul_add(Simd::<f64, N>::splat(0.5), Simd::<f64, N>::splat(0.5)),
    );
    let z = small.select(ax, crate::sqrt_f64(z2));
    let p = (z * z2).mul_add(asin_poly_f64(z2), z).copysign(x);
    let offset = x.simd_lt(Simd::<f64, N>::splat(0.0)).select(
        Simd::<f64, N>::splat(core::f64::consts::PI),
        Simd::<f64, N>::splat(0.0),
    );
    let mul = small.select(Simd::<f64, N>::splat(-1.0), Simd::<f64, N>::splat(2.0));
    let add = small.select(Simd::<f64, N>::splat(core::f64::consts::FRAC_PI_2), offset);
    p.mul_add(mul, add)
}
