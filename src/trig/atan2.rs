/*
 * Adapted from musl libc: atan2.c and atan2f.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::atan::{atan_f32, atan_f64};
use super::data::*;

macro_rules! make_helpers {
    ($name:ident, $float:ident, $uint:ident, $atan:ident, $word_shift:expr,
     $exponent_gap:expr, $pi:ident, $pi_lo:ident, $pio2:expr, $pio4:expr) => {
        #[allow(unused_braces)]
        /// Computes atan2(y, x) in radians for each lane, assuming round-to-nearest, ties-to-even.
        #[inline]
        pub fn $name<const N: usize>(y: Simd<$float, N>, x: Simd<$float, N>) -> Simd<$float, N> {
            vectorize!(N, {
                let ax = x.abs();
                let ay = y.abs();
                let ix = (x.to_bits() >> scalar!($word_shift)) & 0x7fffffff;
                let iy = (y.to_bits() >> scalar!($word_shift)) & 0x7fffffff;
                let negative_x = x.is_sign_negative();
                let large_ratio = ix + scalar!($exponent_gap) < iy;
                let tiny_ratio = negative_x & (iy + scalar!($exponent_gap) < ix);
                let active = x.is_finite()
                    & y.is_finite()
                    & (ax != 0.0)
                    & (ay != 0.0)
                    & !large_ratio
                    & !tiny_ratio;
                // Mask before division so exceptional lanes cannot produce
                // zero denominators or inf/inf in the shared atan kernel.
                let numerator = if active { ay } else { 0.0 };
                let denominator = if active { ax } else { 1.0 };
                let z = $atan(numerator / denominator);
                let result = if negative_x {
                    scalar!($pi) - (z - scalar!($pi_lo))
                } else {
                    z
                };
                let result = if large_ratio | y.is_infinite() | (ax == 0.0) {
                    scalar!($pio2)
                } else {
                    result
                };
                let infinite_x = if y.is_infinite() {
                    if negative_x {
                        3.0 * scalar!($pio4)
                    } else {
                        scalar!($pio4)
                    }
                } else {
                    if negative_x {
                        scalar!($pi)
                    } else {
                        0.0
                    }
                };
                let result = if x.is_infinite() { infinite_x } else { result };
                let zero_y = if negative_x { scalar!($pi) } else { 0.0 };
                let result = if ay == 0.0 { zero_y } else { result };
                let sign = y.to_bits() & scalar!((1 as $uint) << ($uint::BITS - 1));
                let result = <$float>::from_bits(result.to_bits() ^ sign);
                if x.is_nan() | y.is_nan() {
                    x + y
                } else {
                    result
                }
            })
        }
    };
}

make_helpers!(
    atan2_f32,
    f32,
    u32,
    atan_f32,
    0u32,
    26u32 << 23,
    PI_F32,
    PI_LO_F32,
    f32::from_bits(0x3fc90fdb),
    f32::from_bits(0x3f490fdb)
);
make_helpers!(
    atan2_f64,
    f64,
    u64,
    atan_f64,
    32u64,
    64u64 << 20,
    PI,
    PI_LO,
    PIO2,
    PIO4
);
