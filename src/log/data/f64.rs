/*
 * Derived from musl src/math/log1p.c and log10.c, with changes.
 * SPDX-License-Identifier: MIT AND SunPro
 */

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

pub(crate) const SERIES_F64: [f64; 7] = [
    f64::from_bits(0x3fe5555555555593),
    f64::from_bits(0x3fd999999997fa04),
    f64::from_bits(0x3fd2492494229359),
    f64::from_bits(0x3fcc71c51d8e78af),
    f64::from_bits(0x3fc7466496cb03de),
    f64::from_bits(0x3fc39a09d078c69f),
    f64::from_bits(0x3fc2f112df3e5244),
];

pub(crate) const SERIES_LN2_HI_F64: f64 = f64::from_bits(0x3fe62e42fee00000);

pub(crate) const SERIES_LN2_LO_F64: f64 = f64::from_bits(0x3dea39ef35793c76);

pub(crate) const INV_LN10_HI_F64: f64 = f64::from_bits(0x3fdbcb7b15200000);

pub(crate) const INV_LN10_LO_F64: f64 = f64::from_bits(0x3dbb9438ca9aadd5);

pub(crate) const LOG10_2_HI_F64: f64 = f64::from_bits(0x3fd34413509f6000);

pub(crate) const LOG10_2_LO_F64: f64 = f64::from_bits(0x3d59fef311f12b36);
