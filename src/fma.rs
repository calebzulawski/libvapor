/*
 * Adapted from musl libc
 * Copyright © 2005-2020 Rich Felker, et al.
 * SPDX-License-Identifier: MIT
 *
 * Copyright (c) 2005-2011 David Schultz <das@FreeBSD.ORG>
 * SPDX-License-Identifier: BSD-2-Clause
 */

use core::simd::{prelude::*, LaneCount, SupportedLaneCount};
use simd_macros::vectorize;

macro_rules! make_f32_fns {
    { $($ty:ident, $len:literal)* } => {
        $(
        paste::paste! {
            #[no_mangle]
            pub fn [<vapor_fma_ $ty>](x: $ty, y: $ty, z: $ty) -> $ty {
                vectorize!($len, {
                    let x = x as f64;
                    let y = y as f64;
                    let z = z as f64;
                    let xy = x * y;
                    let result = xy + z;

                    let halfway = result.to_bits() & 0x1fffffff == 0x10000000;
                    let exact = (result - xy == z) & (result - z == xy);
                    if (!halfway | result.is_nan() | exact).cast() {
                        result as f32
                    } else {
                        let err = if result.is_sign_negative() == (z > xy) {
                            xy - result + z
                        } else {
                            z - result + xy
                        };
                        if (result.is_sign_negative() == (err < 0.0)).cast() {
                            <f64>::from_bits(result.to_bits() + 1) as f32
                        } else {
                            <f64>::from_bits(result.to_bits() - 1) as f32
                        }
                    }
                })
            }
        }
        )*
    }
}

make_f32_fns! {
    f32x2, 2
    f32x4, 4
    f32x8, 8
}
