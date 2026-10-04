/*
 * Derived from musl src/math/__sindf.c, __cosdf.c, __tandf.c, and __rem_pio2f.c;
 * Arm optimized-routines math/sincosf.h and sincosf_data.c, with changes.
 * SPDX-License-Identifier: MIT AND SunPro AND LicenseRef-SunPro-short
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2018-2024, Arm Limited.
 * Copyright (c) 2018-2019, Arm Limited.
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
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
 * Copyright 2004 Sun Microsystems, Inc.  All Rights Reserved.
 *
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

/*
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
 * Debugged and optimized by Bruce D. Evans.
 */

// Coefficients: musl __sindf.c.
pub(crate) const SIN_POLY_F32: [f64; 4] = [
    f64::from_bits(0xbfc5555554cbac77),
    f64::from_bits(0x3f811110896efbb2),
    f64::from_bits(0xbf2a00f9e2cae774),
    f64::from_bits(0x3ec6cd878c3b46a7),
];

// Coefficients: musl __cosdf.c.
pub(crate) const COS_POLY_F32: [f64; 4] = [
    f64::from_bits(0xbfdffffffd0c5e81),
    f64::from_bits(0x3fa55553e1053a42),
    f64::from_bits(0xbf56c087e80f1e27),
    f64::from_bits(0x3ef99342e0ee5069),
];

// Coefficients: musl __tandf.c.
pub(crate) const TAN_POLY_F32: [f64; 6] = [
    f64::from_bits(0x3fd5554d3418c99f),
    f64::from_bits(0x3fc112fd38999f72),
    f64::from_bits(0x3fab54c91d865afe),
    f64::from_bits(0x3f991df3908c33ce),
    f64::from_bits(0x3f685dadfcecf44e),
    f64::from_bits(0x3f8362b9bf971bcd),
];

// Reduction constants: musl __rem_pio2f.c.
pub(crate) const SHIFT: f64 = f64::from_bits(0x4338000000000000);

pub(crate) const INV_PIO2: f64 = f64::from_bits(0x3fe45f306dc9c883);

pub(crate) const PIO2_REDUCE_HI_F32: f64 = f64::from_bits(0x3ff921fb50000000);

pub(crate) const PIO2_REDUCE_LO_F32: f64 = f64::from_bits(0x3e5110b4611a6263);

// Fixed-point pi/2 scale: Arm optimized-routines sincosf.h.
pub(crate) const PIO2_FIXED_F32: f64 = f64::from_bits(0x3c1921fb54442d18);

// Reduction limit: musl __rem_pio2f.c.
pub(crate) const REDUCTION_LIMIT_F32: f64 = f32::from_bits(0x4dc90fdb) as f64;

// Table: Arm optimized-routines sincosf_data.c.
// 192 bits of 4/pi, with overlapping 32-bit windows spaced eight bits apart.
pub(crate) const INV_PIO4_F32: [u32; 24] = [
    0xa2, 0xa2f9, 0xa2f983, 0xa2f9836e, 0xf9836e4e, 0x836e4e44, 0x6e4e4415, 0x4e441529, 0x441529fc,
    0x1529fc27, 0x29fc2757, 0xfc2757d1, 0x2757d1f5, 0x57d1f534, 0xd1f534dd, 0xf534ddc0, 0x34ddc0db,
    0xddc0db62, 0xc0db6295, 0xdb629599, 0x6295993c, 0x95993c43, 0x993c4390, 0x3c439041,
];
