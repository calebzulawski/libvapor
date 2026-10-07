/*
 * Derived from Arm optimized-routines v23.01 math/v_sinf.c, with changes.
 * SPDX-License-Identifier: MIT
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2022, Arm Limited.
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

#[inline]
pub(super) fn round_quadrant<const N: usize>(ax: Simd<f64, N>) -> (Simd<u64, N>, Simd<f64, N>) {
    // Both fast reducers keep ax*2/pi below 2^29. SHIFT = 1.5*2^52
    // has unit spacing and zero low mantissa bits, so adding it rounds
    // the count and leaves count modulo four in those bits. Read them
    // before subtracting SHIFT to avoid a floating-point-to-u64 cast.
    let shifted = ax * Simd::splat(INV_PIO2) + Simd::splat(SHIFT);
    let quadrant = shifted.to_bits() & Simd::splat(3);
    (quadrant, shifted - Simd::splat(SHIFT))
}
