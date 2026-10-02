/*
 * Adapted from musl libc: log.c and logf.c.
 * Copyright (c) 2017-2018, Arm Limited.
 * SPDX-License-Identifier: MIT
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::natural::*;
use super::data::{INVC_F32, LN2};
use super::reduction::{
    finish_f32, finish_f64, normalize_f32, normalize_f64, table_f32, table_f64,
};

fn log_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let bits = normalize_f32(x);
    let (index, k, z) = table_f32(bits);
    let y = vectorize!(N, {
        let invc = <f64>::gather_or(&INVC_F32, index, 0.0);
        let logc = <f64>::gather_or(&LOGC_F32, index, 0.0);
        let r = z * invc - 1.0;
        let y0 = logc + k * scalar!(LN2);
        let r2 = r * r;
        let y = scalar!(POLY_F32[1]) * r + scalar!(POLY_F32[2]);
        let y = scalar!(POLY_F32[0]) * r2 + y;
        let y = y * r2 + (y0 + r);
        y as f32
    });
    finish_f32(x, y)
}

fn near_one<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let r = x - 1.0;
        let r2 = r * r;
        let r3 = r * r2;
        let y = r3
            * (scalar!(NEAR_POLY_F64[1])
                + r * scalar!(NEAR_POLY_F64[2])
                + r2 * scalar!(NEAR_POLY_F64[3])
                + r3 * (scalar!(NEAR_POLY_F64[4])
                    + r * scalar!(NEAR_POLY_F64[5])
                    + r2 * scalar!(NEAR_POLY_F64[6])
                    + r3 * (scalar!(NEAR_POLY_F64[7])
                        + r * scalar!(NEAR_POLY_F64[8])
                        + r2 * scalar!(NEAR_POLY_F64[9])
                        + r3 * scalar!(NEAR_POLY_F64[10]))));
        let w = r * 134217728.0;
        let rhi = r + w - w;
        let rlo = r - rhi;
        let w = rhi * rhi * scalar!(NEAR_POLY_F64[0]);
        let hi = r + w;
        let lo = r - hi + w;
        let lo = lo + scalar!(NEAR_POLY_F64[0]) * rlo * (rhi + r);
        (y + lo) + hi
    })
}

fn log_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let near = x.simd_ge(Simd::splat(NEAR_LO)) & x.simd_lt(Simd::splat(NEAR_HI));
    if near.all() {
        return finish_f64(x, near_one(x));
    }
    let bits = normalize_f64(x);
    let (index, k, z) = table_f64(bits, 7);
    let mut y = vectorize!(N, {
        let invc = <f64>::gather_or(&INVC_F64, index, 0.0);
        let logc = <f64>::gather_or(&LOGC_F64, index, 0.0);
        let chi = <f64>::gather_or(&CHI_F64, index, 0.0);
        let clo = <f64>::gather_or(&CLO_F64, index, 0.0);
        // musl's split centers preserve the reduction residual without FMA.
        let r = (z - chi - clo) * invc;
        let w = k * scalar!(LN2_HI) + logc;
        let hi = w + r;
        let lo = w - hi + r + k * scalar!(LN2_LO);
        let r2 = r * r;
        lo + r2 * scalar!(POLY_F64[0])
            + r * r2
                * (scalar!(POLY_F64[1])
                    + r * scalar!(POLY_F64[2])
                    + r2 * (scalar!(POLY_F64[3]) + r * scalar!(POLY_F64[4])))
            + hi
    });
    if near.any() {
        let a = near.select(x, Simd::splat(1.0));
        y = near.select(near_one(a), y);
    }
    finish_f64(x, y)
}

macro_rules! make_fns {
    { $($ty:ident, $helper:ident)* } => {
        $(paste::paste! {
            /// Computes the natural logarithm of each lane, assuming round-to-nearest, ties-to-even.
            #[no_mangle]
            pub fn [<vapor_log_ $ty>](x: $ty) -> $ty { $helper(x) }
        })*
    }
}

make_fns! {
    f32x2, log_f32
    f32x4, log_f32
    f32x8, log_f32
    f64x2, log_f64
    f64x4, log_f64
    f64x8, log_f64
}
