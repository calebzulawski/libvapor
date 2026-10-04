/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/asinf.c and acosf.c, with changes.
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
fn asin_poly_f32<const N: usize>(z: Simd<f32, N>) -> Simd<f32, N> {
    let c0 = Simd::splat(f32::from_bits(0x3e2aaaaf)); // 0x1.55555ep-3
    let c1 = Simd::splat(f32::from_bits(0x3d99930d)); // 0x1.33261ap-4
    let c2 = Simd::splat(f32::from_bits(0x3d386bee)); // 0x1.70d7dcp-5
    let c3 = Simd::splat(f32::from_bits(0x3cd82ce8)); // 0x1.b059dp-6
    let c4 = Simd::splat(f32::from_bits(0x3d1d7bec)); // 0x1.3af7d8p-5
    let p0_0 = z.mul_add(c1, c0);
    let p0_1 = z.mul_add(c3, c2);
    let z2 = z * z;
    let p1_0 = z2.mul_add(p0_1, p0_0);
    let z4 = z2 * z2;
    let p2_0 = z4.mul_add(c4, p1_0);
    p2_0
}

/// Computes asin(x), assuming round-to-nearest, ties-to-even.
#[inline]
pub fn asin_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    let small = ax.simd_lt(Simd::<f32, N>::splat(0.5));
    let z2 = small.select(
        x * x,
        (-ax).mul_add(Simd::<f32, N>::splat(0.5), Simd::<f32, N>::splat(0.5)),
    );
    let z = small.select(ax, crate::sqrt_f32(z2));
    let p = (z * z2).mul_add(asin_poly_f32(z2), z);
    small
        .select(
            p,
            p.mul_add(
                Simd::<f32, N>::splat(-2.0),
                Simd::<f32, N>::splat(core::f32::consts::FRAC_PI_2),
            ),
        )
        .copysign(x)
}

/// Computes acos(x), assuming round-to-nearest, ties-to-even.
#[inline]
pub fn acos_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    let small = ax.simd_le(Simd::<f32, N>::splat(0.5));
    let z2 = small.select(
        x * x,
        (-ax).mul_add(Simd::<f32, N>::splat(0.5), Simd::<f32, N>::splat(0.5)),
    );
    let z = small.select(ax, crate::sqrt_f32(z2));
    let p = (z * z2).mul_add(asin_poly_f32(z2), z).copysign(x);
    let offset = x.simd_lt(Simd::<f32, N>::splat(0.0)).select(
        Simd::<f32, N>::splat(core::f32::consts::PI),
        Simd::<f32, N>::splat(0.0),
    );
    let mul = small.select(Simd::<f32, N>::splat(-1.0), Simd::<f32, N>::splat(2.0));
    let add = small.select(Simd::<f32, N>::splat(core::f32::consts::FRAC_PI_2), offset);
    p.mul_add(mul, add)
}
