/*
 * Adapted from musl libc: log1p.c and log1pf.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::*;
use super::reduction::{interval_f32, interval_f64};
use super::series::{series_f32, series_f64};

macro_rules! make_helpers {
    ($name:ident, $float:ident, $interval:ident, $series:ident,
     $ln2_hi:ident, $ln2_lo:ident, $direct_lo:expr, $direct_hi:expr, $tiny:expr, $correction_limit:expr) => {
        #[allow(unused_braces)]
        fn $name<const N: usize>(x: Simd<$float, N>) -> Simd<$float, N> {
            vectorize!(N, {
                let valid = x.is_finite() & (x > -1.0);
                let a = if valid { x } else { 0.0 };
                let direct = (a >= scalar!($direct_lo)) & (a < scalar!($direct_hi));
                let u = 1.0 + a;
                let (k, f) = $interval(u.to_bits());
                // Account for the low part discarded when forming 1+x.
                // For large k the correction is below the result's precision.
                let residual = if k >= 2.0 {
                    1.0 - (u - a)
                } else {
                    a - (u - 1.0)
                };
                let c = if k < scalar!($correction_limit) {
                    residual / u
                } else {
                    0.0
                };
                let f = if direct { a } else { f };
                let k = if direct { 0.0 } else { k };
                let c = if direct { 0.0 } else { c };
                let (s, hfsq, r) = $series(f);
                let y =
                    s * (hfsq + r) + (k * scalar!($ln2_lo) + c) - hfsq + f + k * scalar!($ln2_hi);
                let y = if a.abs() < scalar!($tiny) { a } else { y };
                if x.is_nan() {
                    x + x
                } else if x < -1.0 {
                    scalar!($float::NAN)
                } else if x == -1.0 {
                    scalar!($float::NEG_INFINITY)
                } else if x.is_infinite() {
                    x
                } else {
                    y
                }
            })
        }
    };
}

make_helpers!(
    log1p_f32,
    f32,
    interval_f32,
    series_f32,
    SERIES_LN2_HI_F32,
    SERIES_LN2_LO_F32,
    f32::from_bits(0xbe95f619),
    f32::from_bits(0x3ed413d0),
    f32::from_bits(0x33800000),
    25.0
);
make_helpers!(
    log1p_f64,
    f64,
    interval_f64,
    series_f64,
    SERIES_LN2_HI_F64,
    SERIES_LN2_LO_F64,
    f64::from_bits(0xbfd2bec4ffffffff),
    f64::from_bits(0x3fda827a00000000),
    f64::from_bits(0x3ca0000000000000),
    54.0
);

macro_rules! make_fns {
    { $($ty:ident, $helper:ident)* } => {
        $(paste::paste! {
            /// Computes log(1+x) accurately near zero for each lane, assuming round-to-nearest, ties-to-even.
            #[no_mangle]
            pub fn [<vapor_log1p_ $ty>](x: $ty) -> $ty { $helper(x) }
        })*
    }
}

make_fns! {
    f32x2, log1p_f32
    f32x4, log1p_f32
    f32x8, log1p_f32
    f64x2, log1p_f64
    f64x4, log1p_f64
    f64x8, log1p_f64
}
