/*
 * Range reduction adapted from Arm optimized-routines math/aarch64/advsimd/log.c
 * and log2.c, with changes.
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
 * Split ln(2) constants derived from musl src/math/log1p.c.
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

// Non-FMA adaptation of the Arm table reduction. Dyadic centers make the
// subtraction exact without a low center part, halving the lookup row size.
#[path = "table_non_fma/centers.rs"]
mod centers;
#[path = "table_non_fma/params.rs"]
mod params;

use crate::log::{data::*, reduction};
use centers::{LOG2_TABLE, LOG_TABLE};
use core::simd::prelude::*;
use params::{INDEX_SHIFT, OFFSET, TABLE_SIZE};

#[inline]
fn kernel<const N: usize, const BASE2: bool>(bits: Simd<u64, N>) -> Simd<f64, N> {
    let off = Simd::splat(OFFSET);
    // Round the table index before extracting the exponent. A carry at the
    // upper endpoint advances k and halves z, keeping the index in range.
    let u = bits - off + Simd::splat(1_u64 << (INDEX_SHIFT - 1));
    let k = ((u >> 32).cast::<i32>() >> 20).cast::<f64>();
    let z = Simd::<f64, N>::from_bits(bits - (u & Simd::splat(0xfff0000000000000)));
    let index = (u >> INDEX_SHIFT) & Simd::splat(TABLE_SIZE as u64 - 1);
    let center = Simd::<f64, N>::from_bits(off + (index << INDEX_SHIFT));
    let table = if BASE2 { &LOG2_TABLE } else { &LOG_TABLE };
    let rows = index.cast::<usize>().to_array().map(|i| table[i]);
    let invc = Simd::from_array(core::array::from_fn(|lane| rows[lane][0]));
    let logc = Simd::from_array(core::array::from_fn(|lane| rows[lane][1]));
    // z and center are within a factor of two, so z - center is exact.
    // The center at one also avoids cancellation for inputs near one.
    let r = (z - center) * invc;
    let r2 = r * r;
    // Taylor series through r^7. Here |r| <= 1/256, so the omitted
    // remainder is below 2^-66. Scale coefficients before evaluation
    // for log2 rather than converting the completed logarithm.
    let coefficients = [-0.5, 1.0 / 3.0, -0.25, 0.2, -1.0 / 6.0, 1.0 / 7.0];
    let c = coefficients.map(|c| {
        Simd::splat(if BASE2 {
            c * core::f64::consts::LOG2_E
        } else {
            c
        })
    });
    let p = (c[0] + r * c[1]) + r2 * ((c[2] + r * c[3]) + r2 * (c[4] + r * c[5]));
    let lo = r2 * p;
    if BASE2 {
        k + (logc + (r * Simd::splat(core::f64::consts::LOG2_E) + lo))
    } else {
        k * Simd::splat(SERIES_LN2_HI_F64)
            + (logc + (r + (lo + k * Simd::splat(SERIES_LN2_LO_F64))))
    }
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
