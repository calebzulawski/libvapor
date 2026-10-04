/*
 * Derived from musl src/math/__sin.c, __cos.c, __tan.c, atan.c,
 * __rem_pio2.c, and __rem_pio2_large.c, with changes.
 * Pi/2 Dekker split: libvapor.
 * SPDX-License-Identifier: MIT AND SunPro AND LicenseRef-SunPro-short
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
 *
 * Optimized by Bruce D. Evans.
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
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

pub(crate) const SIN_POLY_F64: [f64; 6] = [
    f64::from_bits(0xbfc5555555555549),
    f64::from_bits(0x3f8111111110f8a6),
    f64::from_bits(0xbf2a01a019c161d5),
    f64::from_bits(0x3ec71de357b1fe7d),
    f64::from_bits(0xbe5ae5e68a2b9ceb),
    f64::from_bits(0x3de5d93a5acfd57c),
];

pub(crate) const COS_POLY_F64: [f64; 6] = [
    f64::from_bits(0x3fa555555555554c),
    f64::from_bits(0xbf56c16c16c15177),
    f64::from_bits(0x3efa01a019cb1590),
    f64::from_bits(0xbe927e4f809c52ad),
    f64::from_bits(0x3e21ee9ebdb4b1c4),
    f64::from_bits(0xbda8fae9be8838d4),
];

pub(crate) const TAN_POLY_F64: [f64; 13] = [
    f64::from_bits(0x3fd5555555555563),
    f64::from_bits(0x3fc111111110fe7a),
    f64::from_bits(0x3faba1ba1bb341fe),
    f64::from_bits(0x3f9664f48406d637),
    f64::from_bits(0x3f8226e3e96e8493),
    f64::from_bits(0x3f6d6d22c9560328),
    f64::from_bits(0x3f57dbc8fee08315),
    f64::from_bits(0x3f4344d8f2f26501),
    f64::from_bits(0x3f3026f71a8d1068),
    f64::from_bits(0x3f147e88a03792a6),
    f64::from_bits(0x3f12b80f32f0a7e9),
    f64::from_bits(0xbef375cbdb605373),
    f64::from_bits(0x3efb2a7074bf7ad4),
];

pub(crate) const PIO4: f64 = f64::from_bits(0x3fe921fb54442d18);

pub(crate) const PIO4_LO: f64 = f64::from_bits(0x3c81a62633145c07);

pub(crate) const PIO2: f64 = f64::from_bits(0x3ff921fb54442d18);

pub(crate) const PIO2_TAIL: f64 = f64::from_bits(0x3c91a62633145c07);

pub(crate) const PIO2_SPLIT_HI: f64 = f64::from_bits(0x3ff921fb58000000);

pub(crate) const PIO2_SPLIT_LO: f64 = f64::from_bits(0xbe4dde9740000000);

pub(crate) const TAN_TRANSFORM_LIMIT: f64 = f64::from_bits(0x3fe5942800000000);

pub(crate) const PIO2_1: f64 = f64::from_bits(0x3ff921fb54400000);

pub(crate) const PIO2_1T: f64 = f64::from_bits(0x3dd0b4611a626331);

pub(crate) const PIO2_2: f64 = f64::from_bits(0x3dd0b4611a600000);

pub(crate) const PIO2_2T: f64 = f64::from_bits(0x3ba3198a2e037073);

pub(crate) const PIO2_3: f64 = f64::from_bits(0x3ba3198a2e000000);

pub(crate) const PIO2_3T: f64 = f64::from_bits(0x397b839a252049c1);

pub(crate) const REDUCTION_LIMIT_F64: f64 = f64::from_bits(0x413921fb00000000);

// Three leading zero words allow the same gather formula at the medium/large
// boundary, where the 288-bit window extends before the binary point.
pub(crate) const TWO_OVER_PI_F64: [u32; 51] = [
    0, 0, 0, 0xa2f9836e, 0x4e441529, 0xfc2757d1, 0xf534ddc0, 0xdb629599, 0x3c439041, 0xfe5163ab,
    0xdebbc561, 0xb7246e3a, 0x424dd2e0, 0x06492eea, 0x09d1921c, 0xfe1deb1c, 0xb129a73e, 0xe88235f5,
    0x2ebb4484, 0xe99c7026, 0xb45f7e41, 0x3991d639, 0x835339f4, 0x9c845f8b, 0xbdf9283b, 0x1ff897ff,
    0xde05980f, 0xef2f118b, 0x5a0a6d1f, 0x6d367ecf, 0x27cb09b7, 0x4f463f66, 0x9e5fea2d, 0x7527bac7,
    0xebe5f17b, 0x3d0739f7, 0x8a5292ea, 0x6bfb5fb1, 0x1f8d5d08, 0x56033046, 0xfc7b6bab, 0xf0cfbc20,
    0x9af4361d, 0xa9e39161, 0x5ee61b08, 0x6599855f, 0x14a06840, 0x8dffd880, 0x4d732731, 0x06061556,
    0xca73a8c9,
];
