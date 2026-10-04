/*
 * Derived from musl src/math/log1pf.c, with changes.
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

pub(crate) const SERIES_F32: [f32; 4] = [
    f32::from_bits(0x3f2aaaaa),
    f32::from_bits(0x3eccce13),
    f32::from_bits(0x3e91e9ee),
    f32::from_bits(0x3e789e26),
];

pub(crate) const SERIES_LN2_HI_F32: f32 = f32::from_bits(0x3f317180);

pub(crate) const SERIES_LN2_LO_F32: f32 = f32::from_bits(0x3717f7d1);
