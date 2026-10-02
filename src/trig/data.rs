/*
 * Coefficients adapted from musl libc.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Copyright 2004 Sun Microsystems, Inc. All Rights Reserved.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 *
 * The 4/pi table is adapted from Arm optimized-routines: sincosf_data.c.
 * Copyright (c) 2018-2019, Arm Limited.
 * SPDX-License-Identifier: MIT
 */

pub(super) const SIN_POLY_F32: [f64; 4] = [
    f64::from_bits(0xbfc5555554cbac77),
    f64::from_bits(0x3f811110896efbb2),
    f64::from_bits(0xbf2a00f9e2cae774),
    f64::from_bits(0x3ec6cd878c3b46a7),
];

pub(super) const COS_POLY_F32: [f64; 4] = [
    f64::from_bits(0xbfdffffffd0c5e81),
    f64::from_bits(0x3fa55553e1053a42),
    f64::from_bits(0xbf56c087e80f1e27),
    f64::from_bits(0x3ef99342e0ee5069),
];

pub(super) const TAN_POLY_F32: [f64; 6] = [
    f64::from_bits(0x3fd5554d3418c99f),
    f64::from_bits(0x3fc112fd38999f72),
    f64::from_bits(0x3fab54c91d865afe),
    f64::from_bits(0x3f991df3908c33ce),
    f64::from_bits(0x3f685dadfcecf44e),
    f64::from_bits(0x3f8362b9bf971bcd),
];

pub(super) const SHIFT: f64 = f64::from_bits(0x4338000000000000);
pub(super) const INV_PIO2: f64 = f64::from_bits(0x3fe45f306dc9c883);
pub(super) const PIO2_REDUCE_HI_F32: f64 = f64::from_bits(0x3ff921fb50000000);
pub(super) const PIO2_REDUCE_LO_F32: f64 = f64::from_bits(0x3e5110b4611a6263);
pub(super) const PIO2_FIXED_F32: f64 = f64::from_bits(0x3c1921fb54442d18);
pub(super) const REDUCTION_LIMIT_F32: f64 = f32::from_bits(0x4dc90fdb) as f64;

// 192 bits of 4/pi, with overlapping 32-bit windows spaced eight bits apart.
pub(super) const INV_PIO4_F32: [u32; 24] = [
    0xa2, 0xa2f9, 0xa2f983, 0xa2f9836e, 0xf9836e4e, 0x836e4e44, 0x6e4e4415, 0x4e441529, 0x441529fc,
    0x1529fc27, 0x29fc2757, 0xfc2757d1, 0x2757d1f5, 0x57d1f534, 0xd1f534dd, 0xf534ddc0, 0x34ddc0db,
    0xddc0db62, 0xc0db6295, 0xdb629599, 0x6295993c, 0x95993c43, 0x993c4390, 0x3c439041,
];

pub(super) const SIN_POLY_F64: [f64; 6] = [
    f64::from_bits(0xbfc5555555555549),
    f64::from_bits(0x3f8111111110f8a6),
    f64::from_bits(0xbf2a01a019c161d5),
    f64::from_bits(0x3ec71de357b1fe7d),
    f64::from_bits(0xbe5ae5e68a2b9ceb),
    f64::from_bits(0x3de5d93a5acfd57c),
];

pub(super) const COS_POLY_F64: [f64; 6] = [
    f64::from_bits(0x3fa555555555554c),
    f64::from_bits(0xbf56c16c16c15177),
    f64::from_bits(0x3efa01a019cb1590),
    f64::from_bits(0xbe927e4f809c52ad),
    f64::from_bits(0x3e21ee9ebdb4b1c4),
    f64::from_bits(0xbda8fae9be8838d4),
];

pub(super) const TAN_POLY_F64: [f64; 13] = [
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

pub(super) const PIO4: f64 = f64::from_bits(0x3fe921fb54442d18);
pub(super) const PI: f64 = f64::from_bits(0x400921fb54442d18);
pub(super) const PI_LO: f64 = f64::from_bits(0x3ca1a62633145c07);
pub(super) const PI_F32: f32 = f32::from_bits(0x40490fdb);
pub(super) const PI_LO_F32: f32 = f32::from_bits(0xb3bbbd2e);
pub(super) const PIO2_HI_F32: f32 = f32::from_bits(0x3fc90fda);
pub(super) const PIO2_LO_F32: f32 = f32::from_bits(0x33a22168);
pub(super) const PIO4_LO: f64 = f64::from_bits(0x3c81a62633145c07);
pub(super) const TAN_TRANSFORM_LIMIT: f64 = f64::from_bits(0x3fe5942800000000);
pub(super) const PIO2: f64 = f64::from_bits(0x3ff921fb54442d18);
pub(super) const PIO2_TAIL: f64 = f64::from_bits(0x3c91a62633145c07);
pub(super) const PIO2_SPLIT_HI: f64 = f64::from_bits(0x3ff921fb58000000);
pub(super) const PIO2_SPLIT_LO: f64 = f64::from_bits(0xbe4dde9740000000);
pub(super) const PIO2_1: f64 = f64::from_bits(0x3ff921fb54400000);
pub(super) const PIO2_1T: f64 = f64::from_bits(0x3dd0b4611a626331);
pub(super) const PIO2_2: f64 = f64::from_bits(0x3dd0b4611a600000);
pub(super) const PIO2_2T: f64 = f64::from_bits(0x3ba3198a2e037073);
pub(super) const PIO2_3: f64 = f64::from_bits(0x3ba3198a2e000000);
pub(super) const PIO2_3T: f64 = f64::from_bits(0x397b839a252049c1);
pub(super) const REDUCTION_LIMIT_F64: f64 = f64::from_bits(0x413921fb00000000);

// The first 1536 bits of musl's 2/pi table, packed into big-endian u32 words.
// Three leading zero words allow the same gather formula at the medium/large
// boundary, where the 288-bit window extends before the binary point.
pub(super) const TWO_OVER_PI_F64: [u32; 51] = [
    0, 0, 0, 0xa2f9836e, 0x4e441529, 0xfc2757d1, 0xf534ddc0, 0xdb629599, 0x3c439041, 0xfe5163ab,
    0xdebbc561, 0xb7246e3a, 0x424dd2e0, 0x06492eea, 0x09d1921c, 0xfe1deb1c, 0xb129a73e, 0xe88235f5,
    0x2ebb4484, 0xe99c7026, 0xb45f7e41, 0x3991d639, 0x835339f4, 0x9c845f8b, 0xbdf9283b, 0x1ff897ff,
    0xde05980f, 0xef2f118b, 0x5a0a6d1f, 0x6d367ecf, 0x27cb09b7, 0x4f463f66, 0x9e5fea2d, 0x7527bac7,
    0xebe5f17b, 0x3d0739f7, 0x8a5292ea, 0x6bfb5fb1, 0x1f8d5d08, 0x56033046, 0xfc7b6bab, 0xf0cfbc20,
    0x9af4361d, 0xa9e39161, 0x5ee61b08, 0x6599855f, 0x14a06840, 0x8dffd880, 0x4d732731, 0x06061556,
    0xca73a8c9,
];

pub(super) const ATAN_HI_F32: [f32; 4] = [
    f32::from_bits(0x3eed6338),
    f32::from_bits(0x3f490fda),
    f32::from_bits(0x3f7b985e),
    PIO2_HI_F32,
];

pub(super) const ATAN_LO_F32: [f32; 4] = [
    f32::from_bits(0x31ac3769),
    f32::from_bits(0x33222168),
    f32::from_bits(0x33140fb4),
    PIO2_LO_F32,
];

pub(super) const ATAN_POLY_F32: [f32; 5] = [
    f32::from_bits(0x3eaaaaa9),
    f32::from_bits(0xbe4cca98),
    f32::from_bits(0x3e11f50d),
    f32::from_bits(0xbdda1247),
    f32::from_bits(0x3d7cac25),
];

pub(super) const ATAN_HI_F64: [f64; 4] = [
    f64::from_bits(0x3fddac670561bb4f),
    f64::from_bits(0x3fe921fb54442d18),
    f64::from_bits(0x3fef730bd281f69b),
    f64::from_bits(0x3ff921fb54442d18),
];

pub(super) const ATAN_LO_F64: [f64; 4] = [
    f64::from_bits(0x3c7a2b7f222f65e2),
    f64::from_bits(0x3c81a62633145c07),
    f64::from_bits(0x3c7007887af0cbbd),
    f64::from_bits(0x3c91a62633145c07),
];

pub(super) const ATAN_POLY_F64: [f64; 11] = [
    f64::from_bits(0x3fd555555555550d),
    f64::from_bits(0xbfc999999998ebc4),
    f64::from_bits(0x3fc24924920083ff),
    f64::from_bits(0xbfbc71c6fe231671),
    f64::from_bits(0x3fb745cdc54c206e),
    f64::from_bits(0xbfb3b0f2af749a6d),
    f64::from_bits(0x3fb10d66a0d03d51),
    f64::from_bits(0xbfadde2d52defd9a),
    f64::from_bits(0x3fa97b4b24760deb),
    f64::from_bits(0xbfa2b4442c6a6c2f),
    f64::from_bits(0x3f90ad3ae322da11),
];

pub(super) const ASIN_NUM_F32: [f32; 3] = [
    f32::from_bits(0x3e2aaa75),
    f32::from_bits(0xbd2f13ba),
    f32::from_bits(0xbc0dd36b),
];

pub(super) const ASIN_DEN_F32: [f32; 1] = [f32::from_bits(0xbf34e5ae)];

pub(super) const ASIN_NUM_F64: [f64; 6] = [
    f64::from_bits(0x3fc5555555555555),
    f64::from_bits(0xbfd4d61203eb6f7d),
    f64::from_bits(0x3fc9c1550e884455),
    f64::from_bits(0xbfa48228b5688f3b),
    f64::from_bits(0x3f49efe07501b288),
    f64::from_bits(0x3f023de10dfdf709),
];

pub(super) const ASIN_DEN_F64: [f64; 4] = [
    f64::from_bits(0xc0033a271c8a2d4b),
    f64::from_bits(0x40002ae59c598ac8),
    f64::from_bits(0xbfe6066c1b8d0159),
    f64::from_bits(0x3fb3b8c5b12e9282),
];
