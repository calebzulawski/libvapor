/*
 * Derived from musl src/math/fma.c (integer implementation), with changes.
 * Copyright and license notices inherited from musl follow.
 */

/*
 * Copyright © 2005-2020 Rich Felker, et al.
 * SPDX-License-Identifier: MIT
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use core::simd::prelude::*;

const SIGN: u64 = 0x8000000000000000;

/// A 54-bit significand and its exponent, including one low guard bit.
/// For finite nonzero inputs, x = m * 2^e, apart from the sign.
#[inline]
fn normalize<const N: usize>(x: Simd<f64, N>) -> (Simd<u64, N>, Simd<i64, N>) {
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & Simd::splat(0x7ff)).cast::<i64>();
    let subnormal = exponent.simd_eq(Simd::splat(0));
    let scaled = (x * Simd::splat(f64::from_bits(0x43e0000000000000))).to_bits(); // 2^63
    let bits = subnormal.select(scaled, bits);
    let exponent = subnormal.select(
        ((scaled >> 52) & Simd::splat(0x7ff)).cast::<i64>() - Simd::splat(63),
        exponent,
    );
    let mantissa = ((bits & Simd::splat(0x000fffffffffffff)) | Simd::splat(1 << 52)) << 1;
    (mantissa, exponent - Simd::splat(1076))
}

/// Multiply two 54-bit significands exactly into two 64-bit words.
#[inline]
fn multiply<const N: usize>(x: Simd<u64, N>, y: Simd<u64, N>) -> (Simd<u64, N>, Simd<u64, N>) {
    let xlo = x & Simd::splat(0xffffffff);
    let ylo = y & Simd::splat(0xffffffff);
    let xhi = x >> 32;
    let yhi = y >> 32;
    let low = xlo * ylo;
    let middle = xlo * yhi + xhi * ylo;
    let lo = low + (middle << 32);
    let hi = xhi * yhi + (middle >> 32) + low.simd_gt(lo).select(Simd::splat(1), Simd::splat(0));
    (hi, lo)
}

#[inline]
pub(super) fn fma_f64<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
) -> Simd<f64, N> {
    if N == 1 {
        let (a, b, c) = (x[0], y[0], z[0]);
        let a_abs = a.to_bits() & !SIGN;
        let b_abs = b.to_bits() & !SIGN;
        let c_abs = c.to_bits() & !SIGN;
        let limit = f64::INFINITY.to_bits() - 1;
        // Combining zero and nonfinite checks keeps LLVM from replacing
        // individual zero bit tests with FP comparisons on subnormals.
        if a_abs.wrapping_sub(1) >= limit || b_abs.wrapping_sub(1) >= limit {
            return Simd::splat(a * b + c);
        }
        if c_abs.wrapping_sub(1) >= limit {
            return if c_abs == 0 { Simd::splat(a * b) } else { z };
        }
        return Simd::splat(fma_f64_scalar_nonzero(a, b, c));
    }
    let zero = Simd::splat(0.0);
    let regular = x.is_finite()
        & y.is_finite()
        & z.is_finite()
        & x.simd_ne(zero)
        & y.simd_ne(zero)
        & z.simd_ne(zero);
    if regular.all() {
        fma_f64_nonzero(x, y, z)
    } else {
        fma_f64_general(x, y, z, regular)
    }
}

#[cold]
#[inline(never)]
fn fma_f64_general<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
    regular: Mask<i64, N>,
) -> Simd<f64, N> {
    let one = Simd::splat(1.0);
    let result = fma_f64_nonzero(
        regular.select(x, one),
        regular.select(y, one),
        regular.select(z, one),
    );
    // An infinite addend wins over a finite product, even if multiplying
    // separately would overflow to an infinity with the opposite sign.
    let xy = x * y;
    let special_product =
        !x.is_finite() | !y.is_finite() | x.simd_eq(Simd::splat(0.0)) | y.simd_eq(Simd::splat(0.0));
    let special = special_product.select(xy + z, z.simd_eq(Simd::splat(0.0)).select(xy, z));
    regular.select(result, special)
}

#[inline]
fn fma_f64_nonzero<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
) -> Simd<f64, N> {
    let zero = Simd::splat(0u64);
    let one = Simd::splat(1u64);
    let (mx, ex) = normalize(x);
    let (my, ey) = normalize(y);
    let (mz, ez) = normalize(z);
    let (hi, lo) = multiply(mx, my);
    let e = ex + ey;
    let d = ez - e;

    // Align the addend and product in 128 bits. Bits shifted out are
    // summarized by a sticky bit, so they still affect the final rounding.
    // Clamp every shift because SIMD selects evaluate both alternatives.
    let left = d.simd_gt(Simd::splat(0));
    let above = d.simd_ge(Simd::splat(64));
    let k = d.simd_clamp(Simd::splat(1), Simd::splat(63)).cast::<u64>();
    let kr = (-d)
        .simd_clamp(Simd::splat(1), Simd::splat(63))
        .cast::<u64>();
    let zright = (mz >> kr)
        | (mz << (Simd::splat(64) - kr))
            .simd_ne(zero)
            .select(one, zero);
    let zright = d
        .simd_eq(Simd::splat(0))
        .select(mz, d.simd_gt(Simd::splat(-64)).select(zright, one));
    let zlo = above.select(zero, left.select(mz << k, zright));
    let zhi = above.select(mz, left.select(mz >> (Simd::splat(64) - k), zero));

    let kp = (d - Simd::splat(64))
        .simd_clamp(Simd::splat(1), Simd::splat(63))
        .cast::<u64>();
    let shifted_lo = (hi << (Simd::splat(64) - kp))
        | (lo >> kp)
        | (lo << (Simd::splat(64) - kp))
            .simd_ne(zero)
            .select(one, zero);
    let shift_product = d.simd_gt(Simd::splat(64));
    let fits = d.simd_lt(Simd::splat(128));
    let lo = shift_product.select(fits.select(shifted_lo, one), lo);
    let hi = shift_product.select(fits.select(hi >> kp, zero), hi);
    let e = above.select(ez - Simd::splat(64), e);

    // Add or subtract the magnitudes, propagating the carry or borrow.
    let sign = (x.to_bits() ^ y.to_bits()) & Simd::splat(SIGN);
    let same_sign = (sign ^ (z.to_bits() & Simd::splat(SIGN))).simd_eq(zero);
    let add_lo = lo + zlo;
    let add_hi = hi + zhi + add_lo.simd_lt(zlo).select(one, zero);
    let sub_lo = lo - zlo;
    let sub_hi = hi - zhi - lo.simd_lt(zlo).select(one, zero);
    let negative = (sub_hi >> 63).simd_ne(zero);
    let neg_lo = zero - sub_lo;
    let neg_hi = zero - sub_hi - neg_lo.simd_ne(zero).select(one, zero);
    let lo = same_sign.select(add_lo, negative.select(neg_lo, sub_lo));
    let hi = same_sign.select(add_hi, negative.select(neg_hi, sub_hi));
    let sign = sign ^ (!same_sign & negative).select(Simd::splat(SIGN), zero);
    let cancelled = (hi | lo).simd_eq(zero);

    // Normalize to 63 significant bits with the final bit sticky. A nearly
    // cancelling subtraction may leave its entire result in the low word.
    let high_nonzero = hi.simd_ne(zero);
    let high_shift = hi.leading_zeros() - one;
    let high = (hi << high_shift)
        | (lo >> (Simd::splat(64) - high_shift))
        | (lo << high_shift).simd_ne(zero).select(one, zero);
    let low_shift = lo.leading_zeros().cast::<i64>() - Simd::splat(1);
    let low = low_shift.simd_lt(Simd::splat(0)).select(
        (lo >> 1) | (lo & one),
        lo << low_shift.simd_max(Simd::splat(0)).cast::<u64>(),
    );
    let mantissa = high_nonzero.select(high, low);
    let e = e + high_nonzero.select(Simd::splat(64), Simd::splat(0))
        - high_nonzero.select(high_shift.cast::<i64>(), low_shift);

    // Round directly at the destination precision, including subnormal
    // spacing. This avoids a second rounding during exponent scaling.
    let subnormal = e.simd_lt(Simd::splat(-1084));
    let shift = subnormal
        .select(
            (Simd::splat(-1074i64) - e).simd_min(Simd::splat(63)),
            Simd::splat(10),
        )
        .cast::<u64>();
    let rounded = mantissa >> shift;
    let remainder = mantissa & ((one << shift) - one);
    let halfway = one << (shift - one);
    let round_up =
        remainder.simd_gt(halfway) | (remainder.simd_eq(halfway) & (rounded & one).simd_ne(zero));
    let rounded = rounded + round_up.select(one, zero);
    let normal = ((e + Simd::splat(1084)).cast::<u64>() << 52) + rounded;
    let magnitude = subnormal.select(rounded, normal);
    let magnitude = e.simd_lt(Simd::splat(-1137)).select(zero, magnitude);
    let magnitude = e
        .simd_gt(Simd::splat(961))
        .select(Simd::splat(f64::INFINITY.to_bits()), magnitude);
    Simd::from_bits(cancelled.select(zero, sign | magnitude))
}

// Keep the full-range scalar fallback in one function so mixed vectors in
// the double-double path do not spill registers around several inlined
// copies of the integer algorithm.
#[inline(never)]
fn fma_f64_scalar_nonzero(x: f64, y: f64, z: f64) -> f64 {
    fn normalize(bits: u64) -> (u64, i32) {
        let mut exponent = ((bits >> 52) & 0x7ff) as i32;
        let mut mantissa = bits & 0x000fffffffffffff;
        if exponent == 0 {
            let shift = mantissa.leading_zeros() - 11;
            mantissa <<= shift;
            exponent = 1 - shift as i32;
        } else {
            mantissa |= 1 << 52;
        }
        (mantissa << 1, exponent - 1076)
    }

    fn shift_sticky(value: u128, shift: u32) -> u128 {
        if shift < 128 {
            (value >> shift) | ((value << (128 - shift) != 0) as u128)
        } else {
            (value != 0) as u128
        }
    }

    let (mx, ex) = normalize(x.to_bits());
    let (my, ey) = normalize(y.to_bits());
    let (mz, ez) = normalize(z.to_bits());
    let mut product = (mx as u128) * (my as u128);
    let mut exponent = ex + ey;
    let delta = ez - exponent;
    let addend = if delta > 0 {
        if delta < 64 {
            (mz as u128) << delta
        } else {
            if delta > 64 {
                product = shift_sticky(product, (delta - 64) as u32);
            }
            exponent = ez - 64;
            (mz as u128) << 64
        }
    } else if delta < 0 {
        shift_sticky(mz as u128, (-delta) as u32)
    } else {
        mz as u128
    };

    let mut sign = (x.to_bits() ^ y.to_bits()) & SIGN;
    let result = if (sign ^ z.to_bits()) & SIGN == 0 {
        product + addend
    } else if product >= addend {
        product - addend
    } else {
        sign ^= SIGN;
        addend - product
    };
    if result == 0 {
        return 0.0;
    }

    let bits = 128 - result.leading_zeros();
    let mantissa = if bits > 63 {
        let shift = bits - 63;
        exponent += shift as i32;
        shift_sticky(result, shift) as u64
    } else {
        let shift = 63 - bits;
        exponent -= shift as i32;
        (result << shift) as u64
    };
    if exponent > 961 {
        return f64::from_bits(sign | f64::INFINITY.to_bits());
    }
    if exponent < -1137 {
        return f64::from_bits(sign);
    }

    let subnormal = exponent < -1084;
    let shift = if subnormal {
        (-1074 - exponent).min(63)
    } else {
        10
    };
    let rounded = mantissa >> shift;
    let remainder = mantissa & ((1 << shift) - 1);
    let halfway = 1 << (shift - 1);
    let round_up = remainder > halfway || (remainder == halfway && rounded & 1 != 0);
    let rounded = rounded + round_up as u64;
    let magnitude = if subnormal {
        rounded
    } else {
        (((exponent + 1084) as u64) << 52) + rounded
    };
    f64::from_bits(sign | magnitude)
}
