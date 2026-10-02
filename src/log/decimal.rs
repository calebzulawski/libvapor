/*
 * Adapted from musl libc: log10.c and log10f.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::*;
use super::reduction::{
    finish_f32, finish_f64, interval_f32, interval_f64, normalize_f32, normalize_f64,
};
use super::series::series_f64;

fn log10_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let (k, f) = interval_f32(normalize_f32(x));
    // Widen the reconstruction to avoid accumulating multiple f32 rounding errors.
    let y = reconstruct(k.cast(), f.cast());
    finish_f32(x, y.cast())
}

fn log10_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let (k, f) = interval_f64(normalize_f64(x));
    finish_f64(x, reconstruct(k, f))
}

fn reconstruct<const N: usize>(k: Simd<f64, N>, f: Simd<f64, N>) -> Simd<f64, N> {
    let (s, hfsq, r) = series_f64(f);
    vectorize!(N, {
        let hi = f - hfsq;
        let hi = <f64>::from_bits(hi.to_bits() & 0xffffffff00000000);
        let lo = f - hi - hfsq + s * (hfsq + r);
        let val_hi = hi * scalar!(INV_LN10_HI_F64);
        let y = k * scalar!(LOG10_2_HI_F64);
        let val_lo = k * scalar!(LOG10_2_LO_F64)
            + (lo + hi) * scalar!(INV_LN10_LO_F64)
            + lo * scalar!(INV_LN10_HI_F64);
        let w = y + val_hi;
        let val_lo = val_lo + ((y - w) + val_hi);
        val_lo + w
    })
}

macro_rules! make_fns {
    { $($ty:ident, $helper:ident)* } => {
        $(paste::paste! {
            /// Computes the base-ten logarithm of each lane, assuming round-to-nearest, ties-to-even.
            #[no_mangle]
            pub fn [<vapor_log10_ $ty>](x: $ty) -> $ty { $helper(x) }
        })*
    }
}

make_fns! {
    f32x2, log10_f32
    f32x4, log10_f32
    f32x8, log10_f32
    f64x2, log10_f64
    f64x4, log10_f64
    f64x8, log10_f64
}
