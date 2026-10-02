/*
 * Adapted from musl libc logarithm argument reduction.
 * Copyright (c) 2017-2018, Arm Limited.
 * SPDX-License-Identifier: MIT
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

macro_rules! make_helpers {
    ($float:ident, $uint:ident, $int:ident, $normalize:ident, $finish:ident,
     $interval:ident, $scale:expr, $mantissa_bits:expr, $bias:expr, $offset:expr) => {
        #[allow(unused_braces)]
        pub(super) fn $normalize<const N: usize>(x: Simd<$float, N>) -> Simd<$uint, N> {
            vectorize!(N, {
                let valid = x.is_finite() & (x > 0.0);
                let a = if valid { x } else { 1.0 };
                let subnormal = a.is_subnormal();
                // Scale only subnormal lanes. Subtracting the scale's exponent
                // leaves a virtual exponent in wrapping integer arithmetic.
                let scaled = if subnormal { a } else { 1.0 };
                let scaled =
                    scaled * scalar!($float::from_bits(($bias + $scale) << $mantissa_bits));
                let bits = scaled.to_bits() - scalar!($scale << $mantissa_bits);
                if subnormal {
                    bits
                } else {
                    a.to_bits()
                }
            })
        }

        #[allow(unused_braces)]
        pub(super) fn $finish<const N: usize>(
            x: Simd<$float, N>,
            y: Simd<$float, N>,
        ) -> Simd<$float, N> {
            vectorize!(N, {
                if x == 0.0 {
                    scalar!($float::NEG_INFINITY)
                } else if x < 0.0 {
                    scalar!($float::NAN)
                } else if x.is_nan() {
                    x + x
                } else if x.is_infinite() {
                    x
                } else if x == 1.0 {
                    0.0
                } else {
                    y
                }
            })
        }

        pub(super) fn $interval<const N: usize>(
            bits: Simd<$uint, N>,
        ) -> (Simd<$float, N>, Simd<$float, N>) {
            vectorize!(N, {
                // x = 2^k * (1+f), with sqrt(2)/2 <= 1+f < sqrt(2).
                let bits = bits + scalar!(($bias << $mantissa_bits) - $offset);
                let exponent = bits as $int;
                let k = (exponent >> scalar!($mantissa_bits)) - scalar!($bias as $int);
                let fraction_bits = bits & scalar!(((1 as $uint) << $mantissa_bits) - 1);
                let reduced = <$float>::from_bits(fraction_bits + scalar!($offset));
                (k as $float, reduced - 1.0)
            })
        }
    };
}

make_helpers!(
    f32,
    u32,
    i32,
    normalize_f32,
    finish_f32,
    interval_f32,
    23u32,
    23,
    127u32,
    0x3f3504f3u32
);
make_helpers!(
    f64,
    u64,
    i64,
    normalize_f64,
    finish_f64,
    interval_f64,
    52u64,
    52,
    1023u64,
    0x3fe6a09e00000000u64
);

pub(super) fn table_f32<const N: usize>(
    bits: Simd<u32, N>,
) -> (Simd<usize, N>, Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let tmp = bits - 0x3f330000;
        let index = (tmp >> 19) & 15;
        let exponent = tmp as i32;
        let k = exponent >> 23;
        let z = <f32>::from_bits(bits - (tmp & 0xff800000));
        (index as usize, k as f64, z as f64)
    })
}

pub(super) fn table_f64<const N: usize>(
    bits: Simd<u64, N>,
    table_bits: u64,
) -> (Simd<usize, N>, Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let tmp = bits - 0x3fe6000000000000;
        let index = (tmp >> scalar!(52 - table_bits)) & scalar!((1u64 << table_bits) - 1);
        let exponent = tmp as i64;
        let k = exponent >> 52;
        let z = <f64>::from_bits(bits - (tmp & 0xfff0000000000000));
        (index as usize, k as f64, z)
    })
}
