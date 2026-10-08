/*
 * Derived from Arm optimized-routines math/sincosf.h (reduce_large), with changes.
 * SPDX-License-Identifier: MIT
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2018-2024, Arm Limited.
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
use super::Reduced;
use core::simd::prelude::*;

pub(super) fn reduce_large_f32<const N: usize>(bits: Simd<u64, N>) -> Reduced<N> {
    let (index, xi) = {
        let index = (bits >> 26) & Simd::splat(15);
        let index = index.cast::<usize>();
        let shift = (bits >> 23) & Simd::splat(7);
        let xi = ((bits & Simd::splat(0xffffff)) | Simd::splat(0x800000)) << shift;
        (index, xi)
    };
    const TABLE: [[u64; 2]; 16] = {
        let mut table = [[0; 2]; 16];
        let mut i = 0;
        while i < table.len() {
            table[i] = [
                INV_PIO4_F32[i] as u64 | ((INV_PIO4_F32[i + 4] as u64) << 32),
                INV_PIO4_F32[i + 8] as u64,
            ];
            i += 1;
        }
        table
    };
    let (first, third) = crate::table::lookup_pairs(&TABLE, index);
    let (res0, res1, res2) = (
        xi * (first & Simd::splat(0xffffffff)),
        xi * (first >> 32),
        xi * (third & Simd::splat(0xffffffff)),
    );
    let (quadrant, hi) = {
        // Reconstruct the fractional part of x*2/pi with a 32x96-bit product.
        let res0 = (res2 >> 32) | (res0 << 32);
        let res0 = res0 + res1;
        let quadrant = (res0 + (Simd::splat(1u64) << 61)) >> 62;
        let res0 = res0 - (quadrant << 62);
        let hi = res0.cast::<i64>().cast::<f64>() * Simd::splat(PIO2_FIXED_F32);
        (quadrant, hi)
    };
    Reduced {
        quadrant,
        hi,
        lo: Simd::splat(0.0),
    }
}
