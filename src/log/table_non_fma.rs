/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log.c and log2.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2025, Arm Limited.
 * Copyright (c) 2019-2024, Arm Limited.
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

/*
 * Derived from musl src/math/log1p.c and log2.c, with changes.
 * Base conversion: the fdlibm log2.c implementation in musl 1.1.19.
 * SPDX-License-Identifier: MIT AND SunPro
 */

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

// Non-FMA adaptation of the existing Arm table kernels.
// Split table centers make the small-interval reduction accurate without FMA.
#[path = "table_non_fma/centers.rs"]
mod centers;

use super::table_fma::{INVC, LOG2_CENTER, LOG2_POLY, LOG_CENTER, LOG_POLY};
use crate::log::{data::*, reduction};
use centers::*;
use core::simd::prelude::*;

// Interleave each center with its reciprocal and logarithm so every lane
// loads one table row. Build it at compile time from the shared Arm data.
const fn table<const BASE2: bool>() -> [[f64; 4]; 128] {
    let mut rows = [[0.0; 4]; 128];
    let mut i = 0;
    while i < 128 {
        rows[i] = [
            CENTER_HI[i],
            CENTER_LO[i],
            INVC[i],
            if BASE2 { LOG2_CENTER[i] } else { LOG_CENTER[i] },
        ];
        i += 1;
    }
    rows
}

#[inline]
fn kernel<const N: usize, const BASE2: bool>(bits: Simd<u64, N>) -> Simd<f64, N> {
    let u = bits - Simd::splat(0x3fe6900900000000);
    let k = ((u >> 32).cast::<i32>() >> 20).cast::<f64>();
    let z = Simd::<f64, N>::from_bits(bits - (u & Simd::splat(0xfff0000000000000)));
    let index = ((u >> 45) & Simd::splat(127)).cast::<usize>().to_array();
    let table = &const { table::<BASE2>() };
    let rows = index.map(|i| table[i]);
    let get = |field: usize| Simd::from_array(core::array::from_fn(|lane| rows[lane][field]));
    // z and CENTER_HI are within a factor of two, so the first subtraction
    // is exact. CENTER_LO accounts for the rounding of 1/invc.
    let r = ((z - get(0)) - get(1)) * get(2);
    let r2 = r * r;
    let coefficients = if BASE2 { LOG2_POLY } else { LOG_POLY };
    let c = |i: usize| Simd::splat(coefficients[i]);
    let p = (c(0) + r * c(1)) + r2 * ((c(2) + r * c(3)) + r2 * c(4));
    let hi = if BASE2 {
        k + (get(3) + r * Simd::splat(core::f64::consts::LOG2_E))
    } else {
        k * Simd::splat(SERIES_LN2_HI_F64) + (get(3) + (r + k * Simd::splat(SERIES_LN2_LO_F64)))
    };
    hi + r2 * p
}

#[cold]
#[inline(never)]
fn general<const N: usize, const BASE2: bool>(x: Simd<f64, N>) -> Simd<f64, N> {
    let y = kernel::<N, BASE2>(reduction::normalize_f64(x));
    reduction::finish_f64(x, y)
}

#[inline]
fn logarithm<const N: usize, const BASE2: bool>(x: Simd<f64, N>) -> Simd<f64, N> {
    let bits = x.to_bits();
    let high = (bits >> 32).cast::<i32>();
    // Positive normal numbers have high words in [0x00100000, 0x7ff00000).
    // Bias this unsigned range into one signed 32-bit comparison.
    let regular = (high + Simd::splat(i32::MIN.wrapping_sub(0x00100000)))
        .simd_lt(Simd::splat(i32::MIN + 0x7fe00000));
    if regular.all() {
        kernel::<N, BASE2>(bits)
    } else {
        general::<N, BASE2>(x)
    }
}

#[inline]
pub(crate) fn log_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    logarithm::<N, false>(x)
}

#[inline]
pub(crate) fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    logarithm::<N, true>(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log2_powers_of_two() {
        // Check exponent extraction and subnormal normalization exactly;
        // the general accuracy fixtures permit errors of up to four ULP.
        fn check<const N: usize>() {
            for first in (-1074..=1023).step_by(N) {
                let exponents: [i32; N] =
                    std::array::from_fn(|lane| (first + lane as i32).min(1023));
                let x = Simd::from_array(exponents.map(|exponent| {
                    let bits = if exponent < -1022 {
                        1_u64 << (exponent + 1074)
                    } else {
                        ((exponent + 1023) as u64) << 52
                    };
                    f64::from_bits(bits)
                }));
                assert_eq!(log2_f64(x).to_array(), exponents.map(f64::from));
            }
        }
        check::<1>();
        check::<2>();
        check::<3>();
        check::<4>();
        check::<8>();
        check::<16>();
        check::<64>();
    }
}
