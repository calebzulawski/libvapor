/*
 * Adapted from musl libc: exp2f.c, expf.c, exp.c, and exp2.c
 * Copyright (c) 2017-2018, Arm Limited.
 * SPDX-License-Identifier: MIT
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

mod data;
use data::*;

fn exp_poly<const N: usize>(ki: Simd<u64, N>, r: Simd<f64, N>, poly: [f64; 3]) -> Simd<f32, N> {
    vectorize!(N, {
        // TABLE_F32[i] stores the bits of 2^(i/32) minus (i << 47). Adding the
        // shifted rounding bits reconstructs 2^(k/32), including negative k.
        let i = ki & 31;
        let i = i as usize;
        let t = <u64>::gather_or(&TABLE_F32, i, 0);
        let s = <f64>::from_bits(t + (ki << scalar!(52 - TABLE_BITS_F32)));

        let z = scalar!(poly[0]) * r + scalar!(poly[1]);
        let r2 = r * r;
        let y = scalar!(poly[2]) * r + 1.0;
        let y = z * r2 + y;
        let y = y * s;
        y as f32
    })
}

// vectorize! retains the braces of scalar if branches in select arguments.
#[allow(unused_braces)]
fn exp_scale_f64<const N: usize>(
    tmp: Simd<f64, N>,
    sbits: Simd<u64, N>,
    ki: Simd<u64, N>,
    large: Mask<i64, N>,
    scale_exponent: u64,
) -> Simd<f64, N> {
    if !large.any() {
        return vectorize!(N, {
            let scale = <f64>::from_bits(sbits);
            scale + scale * tmp
        });
    }

    vectorize!(N, {
        let one_bits = scalar!(1.0f64.to_bits());
        let positive = ki & 0x80000000 == 0;
        let high = large & positive;
        let low = large & !positive;

        // Keep each inactive path at a valid scale before interpreting its bits
        // as floating point, even when a vector mixes normal and extreme lanes.
        let scale = <f64>::from_bits(if large { one_bits } else { sbits });
        let normal_tmp = if large { 0.0 } else { tmp };
        let normal_result = scale + scale * normal_tmp;

        // Bring an overflowing exponent back into range, then scale the result.
        let high_bits = sbits - scalar!(scale_exponent << 52);
        let scale = <f64>::from_bits(if high { high_bits } else { one_bits });
        let high_tmp = if high { tmp } else { 0.0 };
        let high_scale = scalar!(f64::from_bits((1023 + scale_exponent) << 52));
        let high_result = high_scale * (scale + scale * high_tmp);

        // Compute underflowing results in the normal range. Compensate the sum
        // before scaling down so subnormal results do not suffer double rounding.
        let low_bits = sbits + (1022u64 << 52);
        let scale = <f64>::from_bits(if low { low_bits } else { one_bits });
        let low_tmp = if low { tmp } else { 0.0 };
        let y = scale + scale * low_tmp;
        let lo = scale - y + scale * low_tmp;
        let hi = 1.0 + y;
        let lo = 1.0 - hi + y + lo;
        let rounded = hi + lo - 1.0;
        let y = if y < 1.0 { rounded } else { y };
        let low_result = scalar!(f64::MIN_POSITIVE) * y;

        if high {
            high_result
        } else if low {
            low_result
        } else {
            normal_result
        }
    })
}

fn exp_poly_f64<const N: usize>(
    ki: Simd<u64, N>,
    r: Simd<f64, N>,
    poly: [f64; 5],
    large: Mask<i64, N>,
    scale_exponent: u64,
) -> Simd<f64, N> {
    vectorize!(N, {
        let i = ki & 127;
        let i = i as usize * 2;
        let tail = <f64>::from_bits(<u64>::gather_or(&TABLE_F64, i, 0));
        let sbits = <u64>::gather_or(&TABLE_F64, i + 1, 0) + (ki << scalar!(52 - TABLE_BITS_F64));
        let r2 = r * r;
        let tmp = tail
            + r * scalar!(poly[0])
            + r2 * (scalar!(poly[1]) + r * scalar!(poly[2]))
            + r2 * r2 * (scalar!(poly[3]) + r * scalar!(poly[4]));
        exp_scale_f64(tmp, sbits, ki, large, scale_exponent)
    })
}

/// Computes 2^x for each lane, assuming round-to-nearest, ties-to-even.
#[allow(
    unused_braces,
    unused_parens,
    reason = "vectorize! retains scalar branch braces and grouping"
)]
#[inline]
pub fn exp2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let overflow = x >= 128.0;
        let underflow = x <= -150.0;
        let nan = x.is_nan();

        // Selects evaluate both branches. Keep exceptional lanes out
        // of the reduction and restore their results afterward.
        let xd = (if overflow | underflow | nan { 0.0 } else { x }) as f64;
        let kd = xd + scalar!(EXP2_SHIFT_F32);
        let ki = kd.to_bits();
        let kd = kd - scalar!(EXP2_SHIFT_F32);
        let r = xd - kd;
        let y = exp_poly(ki, r, verbatim!(EXP2_POLY_F32));

        if nan {
            x + x
        } else if overflow {
            scalar!(f32::INFINITY)
        } else if underflow {
            0.0
        } else {
            y
        }
    })
}

/// Computes e^x for each lane, assuming round-to-nearest, ties-to-even.
#[allow(
    unused_braces,
    unused_parens,
    reason = "vectorize! retains scalar branch braces and grouping"
)]
#[inline]
pub fn exp_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let overflow = x > scalar!(f32::from_bits(0x42b17217));
        let underflow = x < scalar!(f32::from_bits(0xc2cff1b4));
        let nan = x.is_nan();
        let xd = (if overflow | underflow | nan { 0.0 } else { x }) as f64;

        // x*32/ln(2) = k + r, with |r| <= 1/2.
        let z = scalar!(INV_LN2_SCALED_F32) * xd;
        let kd = z + scalar!(SHIFT);
        let ki = kd.to_bits();
        let kd = kd - scalar!(SHIFT);
        let r = z - kd;
        let y = exp_poly(ki, r, verbatim!(EXP_POLY_F32));

        if nan {
            x + x
        } else if overflow {
            scalar!(f32::INFINITY)
        } else if underflow {
            0.0
        } else {
            y
        }
    })
}

/// Computes 2^x for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn exp2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let overflow = x >= 1024.0;
        let underflow = x <= -1075.0;
        let nan = x.is_nan();
        let tiny = x.abs() < scalar!(f64::from_bits(0x3c90000000000000));
        let xd = if overflow | underflow | nan | tiny {
            0.0
        } else {
            x
        };

        // x = k/128 + r, with |r| <= 1/256.
        let kd = xd + scalar!(EXP2_SHIFT_F64);
        let ki = kd.to_bits();
        let kd = kd - scalar!(EXP2_SHIFT_F64);
        let r = xd - kd;
        let large = xd.abs() > 928.0;
        let y = exp_poly_f64(ki, r, verbatim!(EXP2_POLY_F64), large, verbatim!(1));

        if nan {
            x + x
        } else if overflow {
            scalar!(f64::INFINITY)
        } else if underflow {
            0.0
        } else if tiny {
            1.0 + x
        } else {
            y
        }
    })
}

/// Computes e^x for each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn exp_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        // Other overflow and underflow cases are handled by scaling.
        let overflow = x >= 1024.0;
        let underflow = x <= -1024.0;
        let nan = x.is_nan();
        let tiny = x.abs() < scalar!(f64::from_bits(0x3c90000000000000));
        let xd = if overflow | underflow | nan | tiny {
            0.0
        } else {
            x
        };

        let z = scalar!(INV_LN2_SCALED_F64) * xd;
        let kd = z + scalar!(SHIFT);
        let ki = kd.to_bits();
        let kd = kd - scalar!(SHIFT);
        // Split ln(2)/128 to preserve precision in the remainder.
        let r = xd + kd * scalar!(NEG_LN2_HI_F64) + kd * scalar!(NEG_LN2_LO_F64);
        let large = xd.abs() >= 512.0;
        let y = exp_poly_f64(ki, r, verbatim!(EXP_POLY_F64), large, verbatim!(1009));

        if nan {
            x + x
        } else if overflow {
            scalar!(f64::INFINITY)
        } else if underflow {
            0.0
        } else if tiny {
            1.0 + x
        } else {
            y
        }
    })
}
