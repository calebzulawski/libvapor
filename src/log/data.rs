/*
 * Adapted from musl libc: log10.c, log10f.c, log1p.c, and log1pf.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

/*
 * Adapted from musl libc logarithm data.
 * Copyright (c) 2017-2018, Arm Limited.
 * SPDX-License-Identifier: MIT
 */

pub(super) mod binary;
pub(super) mod natural;

pub(super) const INVC_F32: [f64; 16] = [
    f64::from_bits(0x3ff661ec79f8f3be),
    f64::from_bits(0x3ff571ed4aaf883d),
    f64::from_bits(0x3ff49539f0f010b0),
    f64::from_bits(0x3ff3c995b0b80385),
    f64::from_bits(0x3ff30d190c8864a5),
    f64::from_bits(0x3ff25e227b0b8ea0),
    f64::from_bits(0x3ff1bb4a4a1a343f),
    f64::from_bits(0x3ff12358f08ae5ba),
    f64::from_bits(0x3ff0953f419900a7),
    f64::from_bits(0x3ff0000000000000),
    f64::from_bits(0x3fee608cfd9a47ac),
    f64::from_bits(0x3feca4b31f026aa0),
    f64::from_bits(0x3feb2036576afce6),
    f64::from_bits(0x3fe9c2d163a1aa2d),
    f64::from_bits(0x3fe886e6037841ed),
    f64::from_bits(0x3fe767dcf5534862),
];

pub(super) const LN2: f64 = f64::from_bits(0x3fe62e42fefa39ef);

pub(super) const SERIES_F32: [f32; 4] = [
    f32::from_bits(0x3f2aaaaa),
    f32::from_bits(0x3eccce13),
    f32::from_bits(0x3e91e9ee),
    f32::from_bits(0x3e789e26),
];

pub(super) const SERIES_LN2_HI_F32: f32 = f32::from_bits(0x3f317180);
pub(super) const SERIES_LN2_LO_F32: f32 = f32::from_bits(0x3717f7d1);

pub(super) const SERIES_F64: [f64; 7] = [
    f64::from_bits(0x3fe5555555555593),
    f64::from_bits(0x3fd999999997fa04),
    f64::from_bits(0x3fd2492494229359),
    f64::from_bits(0x3fcc71c51d8e78af),
    f64::from_bits(0x3fc7466496cb03de),
    f64::from_bits(0x3fc39a09d078c69f),
    f64::from_bits(0x3fc2f112df3e5244),
];

pub(super) const SERIES_LN2_HI_F64: f64 = f64::from_bits(0x3fe62e42fee00000);
pub(super) const SERIES_LN2_LO_F64: f64 = f64::from_bits(0x3dea39ef35793c76);
pub(super) const INV_LN10_HI_F64: f64 = f64::from_bits(0x3fdbcb7b15200000);
pub(super) const INV_LN10_LO_F64: f64 = f64::from_bits(0x3dbb9438ca9aadd5);
pub(super) const LOG10_2_HI_F64: f64 = f64::from_bits(0x3fd34413509f6000);
pub(super) const LOG10_2_LO_F64: f64 = f64::from_bits(0x3d59fef311f12b36);
