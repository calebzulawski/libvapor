/*
 * Adapted from musl libc: log2.c and log2f.c.
 * Copyright (c) 2017-2018, Arm Limited.
 * SPDX-License-Identifier: MIT
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::binary::*;
use super::data::INVC_F32;
use super::reduction::{
    finish_f32, finish_f64, normalize_f32, normalize_f64, table_f32, table_f64,
};

/// Computes the base-two logarithm of each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn log2_f32<const N: usize>(x: Simd<f32, N>) -> Simd<f32, N> {
    let bits = normalize_f32(x);
    let (index, k, z) = table_f32(bits);
    let y = vectorize!(N, {
        let invc = <f64>::gather_or(&INVC_F32, index, 0.0);
        let logc = <f64>::gather_or(&LOGC_F32, index, 0.0);
        let r = z * invc - 1.0;
        let y0 = logc + k;
        let r2 = r * r;
        let y = scalar!(POLY_F32[1]) * r + scalar!(POLY_F32[2]);
        let y = scalar!(POLY_F32[0]) * r2 + y;
        let p = scalar!(POLY_F32[3]) * r + y0;
        let y = y * r2 + p;
        y as f32
    });
    finish_f32(x, y)
}

fn near_one<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let r = x - 1.0;
        let rhi = <f64>::from_bits(r.to_bits() & 0xffffffff00000000);
        let rlo = r - rhi;
        let hi = rhi * scalar!(INV_LN2_HI);
        let lo = rlo * scalar!(INV_LN2_HI) + r * scalar!(INV_LN2_LO);
        let r2 = r * r;
        let r4 = r2 * r2;
        let p = r2 * (scalar!(NEAR_POLY_F64[0]) + r * scalar!(NEAR_POLY_F64[1]));
        let y = hi + p;
        let lo = lo + (hi - y + p);
        let lo = lo
            + r4 * (scalar!(NEAR_POLY_F64[2])
                + r * scalar!(NEAR_POLY_F64[3])
                + r2 * (scalar!(NEAR_POLY_F64[4]) + r * scalar!(NEAR_POLY_F64[5]))
                + r4 * (scalar!(NEAR_POLY_F64[6])
                    + r * scalar!(NEAR_POLY_F64[7])
                    + r2 * (scalar!(NEAR_POLY_F64[8]) + r * scalar!(NEAR_POLY_F64[9]))));
        y + lo
    })
}

/// Computes the base-two logarithm of each lane, assuming round-to-nearest, ties-to-even.
#[inline]
pub fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let near = x.simd_ge(Simd::splat(NEAR_LO)) & x.simd_lt(Simd::splat(NEAR_HI));
    if near.all() {
        return finish_f64(x, near_one(x));
    }
    let bits = normalize_f64(x);
    let (index, k, z) = table_f64(bits, 6);
    let mut y = vectorize!(N, {
        let invc = <f64>::gather_or(&INVC_F64, index, 0.0);
        let logc = <f64>::gather_or(&LOGC_F64, index, 0.0);
        let chi = <f64>::gather_or(&CHI_F64, index, 0.0);
        let clo = <f64>::gather_or(&CLO_F64, index, 0.0);
        let r = (z - chi - clo) * invc;
        let rhi = <f64>::from_bits(r.to_bits() & 0xffffffff00000000);
        let rlo = r - rhi;
        let t1 = rhi * scalar!(INV_LN2_HI);
        let t2 = rlo * scalar!(INV_LN2_HI) + r * scalar!(INV_LN2_LO);
        let t3 = k + logc;
        let hi = t3 + t1;
        let lo = t3 - hi + t1 + t2;
        let r2 = r * r;
        let r4 = r2 * r2;
        let p = scalar!(POLY_F64[0])
            + r * scalar!(POLY_F64[1])
            + r2 * (scalar!(POLY_F64[2]) + r * scalar!(POLY_F64[3]))
            + r4 * (scalar!(POLY_F64[4]) + r * scalar!(POLY_F64[5]));
        lo + r2 * p + hi
    });
    if near.any() {
        let a = near.select(x, Simd::splat(1.0));
        y = near.select(near_one(a), y);
    }
    finish_f64(x, y)
}
