/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/atanf.c and atan2f.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2021-2025, Arm Limited.
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

// The non-FMA variant evaluates every multiply and add separately.
trait UnfusedMulAdd {
    fn mul_add(self, multiplier: Self, addend: Self) -> Self;
}

impl<T, const N: usize> UnfusedMulAdd for Simd<T, N>
where
    T: core::simd::SimdElement,
    Simd<T, N>: core::ops::Mul<Output = Self> + core::ops::Add<Output = Self>,
{
    #[inline]
    fn mul_add(self, multiplier: Self, addend: Self) -> Self {
        self * multiplier + addend
    }
}

#[inline]
fn atan_poly_f32<const N: usize>(z: Simd<f32, N>) -> Simd<f32, N> {
    let c0 = Simd::splat(f32::from_bits(0xbeaaaa6e)); // -0x1.5554dcp-2
    let c1 = Simd::splat(f32::from_bits(0x3e4cbc76)); // 0x1.9978ecp-3
    let c2 = Simd::splat(f32::from_bits(0xbe11854a)); // -0x1.230a94p-3
    let c3 = Simd::splat(f32::from_bits(0x3dda6f58)); // 0x1.b4debp-4
    let c4 = Simd::splat(f32::from_bits(0xbd9aa86d)); // -0x1.3550dap-4
    let c5 = Simd::splat(f32::from_bits(0x3d30f758)); // 0x1.61eebp-5
    let c6 = Simd::splat(f32::from_bits(0xbc860bea)); // -0x1.0c17d4p-6
    let c7 = Simd::splat(f32::from_bits(0x3b3f534a)); // 0x1.7ea694p-9
    let p0_0 = z.mul_add(c1, c0);
    let p0_1 = z.mul_add(c3, c2);
    let p0_2 = z.mul_add(c5, c4);
    let p0_3 = z.mul_add(c7, c6);
    let z2 = z * z;
    let p1_0 = z2.mul_add(p0_1, p0_0);
    let p1_1 = z2.mul_add(p0_3, p0_2);
    let z4 = z2 * z2;
    let p2_0 = z4.mul_add(p1_1, p1_0);
    p2_0
}

/// Computes atan(x), assuming round-to-nearest, ties-to-even.
#[inline]
pub fn atan_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let red = x.abs().simd_gt(Simd::<f32, N>::splat(1.0));
    let den = red.select(x, Simd::<f32, N>::splat(1.0));
    let z = red.select(-Simd::<f32, N>::splat(1.0) / den, x);
    let shift = red
        .select(
            Simd::<f32, N>::splat(core::f32::consts::FRAC_PI_2),
            Simd::<f32, N>::splat(0.0),
        )
        .copysign(x);
    let z2 = z * z;
    let p = atan_poly_f32(z2);
    let y = if core::mem::size_of::<f32>() == 8 {
        let q = (-z2).mul_add(p, Simd::<f32, N>::splat(-1.0));
        (-z).mul_add(q, shift)
    } else {
        (z * z2).mul_add(p, shift + z)
    };
    // Preserve negative zero and the exact tiny-input result.
    let tiny = x.abs().simd_lt(Simd::<f32, N>::splat(1.0e-12));
    tiny.select(x, y)
}

/// Computes atan2(y,x), assuming round-to-nearest, ties-to-even.
#[inline]
pub fn atan2_f32<const N: usize>(y: Simd<f32, N>, x: Simd<f32, N>) -> Simd<f32, N> {
    let ax = x.abs();
    let ay = y.abs();
    let special = !x.is_finite()
        | !y.is_finite()
        | ax.simd_eq(Simd::<f32, N>::splat(0.0))
        | ay.simd_eq(Simd::<f32, N>::splat(0.0));
    let swap = ay.simd_gt(ax);
    let numerator = swap.select(-ax, ay);
    let denominator = special.select(Simd::<f32, N>::splat(1.0), swap.select(ay, ax));
    let z = special.select(Simd::<f32, N>::splat(0.0), numerator) / denominator;
    let shift: Simd<f32, N> = x
        .simd_lt(Simd::<f32, N>::splat(0.0))
        .select(Simd::<f32, N>::splat(-2.0), Simd::<f32, N>::splat(0.0))
        + swap.select(Simd::<f32, N>::splat(1.0), Simd::<f32, N>::splat(0.0));
    let z2 = z * z;
    let p = atan_poly_f32(z2);
    let ret = shift.mul_add(Simd::<f32, N>::splat(core::f32::consts::FRAC_PI_2), z);
    let ret = (z * z2).mul_add(p, ret);
    let sign = (x.to_bits() ^ y.to_bits()) & Simd::<u32, N>::splat(1 << (<u32>::BITS - 1));
    let ret = Simd::from_bits(ret.to_bits() ^ sign);
    if !special.any() {
        return ret;
    }
    // Vector replacements for the upstream scalar exceptional calls.
    let negative_x = x.is_sign_negative();
    let infinite_x = negative_x.select(
        Simd::<f32, N>::splat(core::f32::consts::PI),
        Simd::<f32, N>::splat(0.0),
    );
    let both_infinite = negative_x.select(
        Simd::<f32, N>::splat(3.0 * core::f32::consts::FRAC_PI_4),
        Simd::<f32, N>::splat(core::f32::consts::FRAC_PI_4),
    );
    let infinite_x = y.is_infinite().select(both_infinite, infinite_x);
    let exceptional = x.is_infinite().select(
        infinite_x,
        Simd::<f32, N>::splat(core::f32::consts::FRAC_PI_2),
    );
    let zero_y = negative_x.select(
        Simd::<f32, N>::splat(core::f32::consts::PI),
        Simd::<f32, N>::splat(0.0),
    );
    let exceptional = ay
        .simd_eq(Simd::<f32, N>::splat(0.0))
        .select(zero_y, exceptional)
        .copysign(y);
    let exceptional = (x.is_nan() | y.is_nan()).select(x + y, exceptional);
    special.select(exceptional, ret)
}
