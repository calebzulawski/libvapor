/*
 * Derived from musl src/math/exp.c and exp2.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2018, Arm Limited.
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

use super::super::data::*;
use core::simd::prelude::*;

fn exp_scale_f64<const N: usize>(
    tmp: Simd<f64, N>,
    sbits: Simd<u64, N>,
    ki: Simd<u64, N>,
    large: Mask<i64, N>,
    scale_exponent: u64,
) -> Simd<f64, N> {
    if !large.any() {
        let scale = Simd::<f64, N>::from_bits(sbits);
        return scale + scale * tmp;
    }

    let one_bits = Simd::splat(1.0f64.to_bits());
    let positive = (ki & Simd::splat(0x80000000)).simd_eq(Simd::splat(0));
    let high = large & positive;
    let low = large & !positive;

    // Keep each inactive path at a valid scale before interpreting its bits
    // as floating point, even when a vector mixes normal and extreme lanes.
    let scale = Simd::<f64, N>::from_bits(large.select(one_bits, sbits));
    let normal_tmp = large.select(Simd::splat(0.0), tmp);
    let normal_result = scale + scale * normal_tmp;

    // Bring an overflowing exponent back into range, then scale the result.
    let high_bits = sbits - Simd::splat(scale_exponent << 52);
    let scale = Simd::<f64, N>::from_bits(high.select(high_bits, one_bits));
    let high_tmp = high.select(tmp, Simd::splat(0.0));
    let high_scale = Simd::splat(f64::from_bits((1023 + scale_exponent) << 52));
    let high_result = high_scale * (scale + scale * high_tmp);

    // Compute underflowing results in the normal range. Compensate the sum
    // before scaling down so subnormal results do not suffer double rounding.
    let low_bits = sbits + (Simd::splat(1022u64) << 52);
    let scale = Simd::<f64, N>::from_bits(low.select(low_bits, one_bits));
    let low_tmp = low.select(tmp, Simd::splat(0.0));
    let y = scale + scale * low_tmp;
    let lo = scale - y + scale * low_tmp;
    let hi = Simd::splat(1.0) + y;
    let lo = Simd::splat(1.0) - hi + y + lo;
    let rounded = hi + lo - Simd::splat(1.0);
    let y = y.simd_lt(Simd::splat(1.0)).select(rounded, y);
    let low_result = Simd::splat(f64::MIN_POSITIVE) * y;

    high.select(high_result, low.select(low_result, normal_result))
}

// Inlining keeps the coefficients and scale exponent constant.
#[inline(always)]
fn exp_poly_f64<const N: usize>(
    ki: Simd<u64, N>,
    r: Simd<f64, N>,
    poly: [f64; 5],
    large: Mask<i64, N>,
    scale_exponent: u64,
) -> Simd<f64, N> {
    let index = (ki & Simd::splat(127)).cast::<usize>();
    let (table, _) = TABLE_F64.as_chunks::<2>();
    let (tail, scale) = crate::table::lookup_pairs(table, index);
    let tail = Simd::<f64, N>::from_bits(tail);
    let sbits = scale + (ki << (52 - TABLE_BITS_F64));

    let r2 = r * r;
    let tmp = tail
        + r * Simd::splat(poly[0])
        + r2 * (Simd::splat(poly[1]) + r * Simd::splat(poly[2]))
        + r2 * r2 * (Simd::splat(poly[3]) + r * Simd::splat(poly[4]));
    exp_scale_f64(tmp, sbits, ki, large, scale_exponent)
}

/// Computes 2^x for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn exp2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let overflow = x.simd_ge(Simd::splat(1024.0));
    let underflow = x.simd_le(Simd::splat(-1075.0));
    let nan = x.is_nan();
    let tiny = x
        .abs()
        .simd_lt(Simd::splat(f64::from_bits(0x3c90000000000000)));
    let xd = (overflow | underflow | nan | tiny).select(Simd::splat(0.0), x);

    // x = k/128 + r, with |r| <= 1/256.
    let kd = xd + Simd::splat(EXP2_SHIFT_F64);
    let ki = kd.to_bits();
    let kd = kd - Simd::splat(EXP2_SHIFT_F64);
    let r = xd - kd;
    let large = xd.abs().simd_gt(Simd::splat(928.0));
    let y = exp_poly_f64(ki, r, EXP2_POLY_F64, large, 1);

    nan.select(
        x + x,
        overflow.select(
            Simd::splat(f64::INFINITY),
            underflow.select(Simd::splat(0.0), tiny.select(Simd::splat(1.0) + x, y)),
        ),
    )
}

/// Computes e^x for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn exp_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    // Other overflow and underflow cases are handled by scaling.
    let overflow = x.simd_ge(Simd::splat(1024.0));
    let underflow = x.simd_le(Simd::splat(-1024.0));
    let nan = x.is_nan();
    let tiny = x
        .abs()
        .simd_lt(Simd::splat(f64::from_bits(0x3c90000000000000)));
    let xd = (overflow | underflow | nan | tiny).select(Simd::splat(0.0), x);

    let z = Simd::splat(INV_LN2_SCALED_F64) * xd;
    let kd = z + Simd::splat(SHIFT);
    let ki = kd.to_bits();
    let kd = kd - Simd::splat(SHIFT);
    // Split ln(2)/128 to preserve precision in the remainder.
    let r = xd + kd * Simd::splat(NEG_LN2_HI_F64) + kd * Simd::splat(NEG_LN2_LO_F64);
    let large = xd.abs().simd_ge(Simd::splat(512.0));
    let y = exp_poly_f64(ki, r, EXP_POLY_F64, large, 1009);

    nan.select(
        x + x,
        overflow.select(
            Simd::splat(f64::INFINITY),
            underflow.select(Simd::splat(0.0), tiny.select(Simd::splat(1.0) + x, y)),
        ),
    )
}
