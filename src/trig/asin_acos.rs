/*
 * Adapted from musl libc: asin.c, asinf.c, acos.c, and acosf.c.
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice is preserved.
 */

use core::simd::prelude::*;
use simd_macros::vectorize;

use super::data::*;

fn rational_f32<const N: usize>(z: Simd<f32, N>) -> Simd<f32, N> {
    vectorize!(N, {
        let p = z
            * (scalar!(ASIN_NUM_F32[0])
                + z * (scalar!(ASIN_NUM_F32[1]) + z * scalar!(ASIN_NUM_F32[2])));
        let q = 1.0 + z * scalar!(ASIN_DEN_F32[0]);
        p / q
    })
}

fn rational_f64<const N: usize>(z: Simd<f64, N>) -> Simd<f64, N> {
    vectorize!(N, {
        let p = z
            * (scalar!(ASIN_NUM_F64[0])
                + z * (scalar!(ASIN_NUM_F64[1])
                    + z * (scalar!(ASIN_NUM_F64[2])
                        + z * (scalar!(ASIN_NUM_F64[3])
                            + z * (scalar!(ASIN_NUM_F64[4]) + z * scalar!(ASIN_NUM_F64[5]))))));
        let q = 1.0
            + z * (scalar!(ASIN_DEN_F64[0])
                + z * (scalar!(ASIN_DEN_F64[1])
                    + z * (scalar!(ASIN_DEN_F64[2]) + z * scalar!(ASIN_DEN_F64[3]))));
        p / q
    })
}

#[allow(unused_braces)]
fn split_root_f32<const N: usize>(
    z: Simd<f32, N>,
    s: Simd<f32, N>,
) -> (Simd<f32, N>, Simd<f32, N>) {
    vectorize!(N, {
        let f = <f32>::from_bits(s.to_bits() & 0xfffff000);
        let denominator = if s == 0.0 { 1.0 } else { s + f };
        let c = (z - f * f) / denominator;
        (f, c)
    })
}

#[allow(unused_braces)]
fn split_root_f64<const N: usize>(
    z: Simd<f64, N>,
    s: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        // f+c retains the root's residual; z = 0 is an exact endpoint.
        let f = <f64>::from_bits(s.to_bits() & 0xffffffff00000000);
        let denominator = if s == 0.0 { 1.0 } else { s + f };
        let c = (z - f * f) / denominator;
        (f, c)
    })
}

#[allow(unused_braces)]
fn asin_f32<const N: usize>(
    x: Simd<f32, N>,
    sqrt: impl Fn(Simd<f64, N>) -> Simd<f64, N>,
) -> Simd<f32, N> {
    let (a, valid, upper, z, r, mut result) = vectorize!(N, {
        let ax = x.abs();
        let valid = ax <= 1.0;
        let a = if valid { ax } else { 0.0 };
        let upper = a >= 0.5;
        let z = if upper { (1.0 - a) * 0.5 } else { a * a };
        let r = rational_f32(z);
        (a, valid, upper, z, r, a + a * r)
    });
    if upper.any() {
        // musl's float asin uses a double square root in this interval.
        let s = sqrt(upper.select(z, Simd::splat(1.0)).cast());
        let reduced = vectorize!(N, {
            let result = scalar!(PIO2) - 2.0 * (s + s * r as f64);
            result as f32
        });
        result = upper.select(reduced, result);
    }
    vectorize!(N, {
        let result = if a < scalar!(f32::from_bits(0x39800000)) {
            a
        } else {
            result
        };
        let sign = x.to_bits() & 0x80000000;
        let result = <f32>::from_bits(result.to_bits() ^ sign);
        if valid {
            result
        } else {
            scalar!(f32::NAN)
        }
    })
}

#[allow(unused_braces)]
fn asin_f64<const N: usize>(
    x: Simd<f64, N>,
    sqrt: impl Fn(Simd<f64, N>) -> Simd<f64, N>,
) -> Simd<f64, N> {
    let (a, valid, upper, z, r, mut result) = vectorize!(N, {
        let ax = x.abs();
        let valid = ax <= 1.0;
        let a = if valid { ax } else { 0.0 };
        let upper = a >= 0.5;
        let z = if upper { (1.0 - a) * 0.5 } else { a * a };
        let r = rational_f64(z);
        (a, valid, upper, z, r, a + a * r)
    });
    if upper.any() {
        let s = sqrt(upper.select(z, Simd::splat(1.0)));
        let (f, c) = split_root_f64(z, s);
        let reduced = vectorize!(N, {
            let near_one = a >= scalar!(f64::from_bits(0x3fef333300000000));
            if near_one {
                scalar!(PIO2) - (2.0 * (s + s * r) - scalar!(PIO2_TAIL))
            } else {
                scalar!(PIO4)
                    - (2.0 * s * r - (scalar!(PIO2_TAIL) - 2.0 * c) - (scalar!(PIO4) - 2.0 * f))
            }
        });
        result = upper.select(reduced, result);
    }
    vectorize!(N, {
        let result = if a < scalar!(f64::from_bits(0x3e50000000000000)) {
            a
        } else {
            result
        };
        let sign = x.to_bits() & (1u64 << 63);
        let result = <f64>::from_bits(result.to_bits() ^ sign);
        if valid {
            result
        } else {
            scalar!(f64::NAN)
        }
    })
}

#[allow(unused_braces)]
fn acos_f32<const N: usize>(
    x: Simd<f32, N>,
    sqrt: impl Fn(Simd<f32, N>) -> Simd<f32, N>,
) -> Simd<f32, N> {
    let (valid, upper, z, r, mut result) = vectorize!(N, {
        let valid = x.abs() <= 1.0;
        let a = if valid { x } else { 0.0 };
        let upper = a.abs() >= 0.5;
        let z = if upper { (1.0 - a.abs()) * 0.5 } else { a * a };
        let r = rational_f32(z);
        let result = scalar!(PIO2_HI_F32) - (a - (scalar!(PIO2_LO_F32) - a * r));
        (valid, upper, z, r, result)
    });
    if upper.any() {
        let s = sqrt(upper.select(z, Simd::splat(1.0)));
        let (f, c) = split_root_f32(z, s);
        let reduced = vectorize!(N, {
            if x.is_sign_negative() {
                let w = r * s - scalar!(PIO2_LO_F32);
                2.0 * (scalar!(PIO2_HI_F32) - (s + w))
            } else {
                // Reconstruct directly near +1 to avoid pi/2 - asin(x)
                // cancellation when acos(x) is small.
                let w = r * s + c;
                2.0 * (f + w)
            }
        });
        result = upper.select(reduced, result);
    }
    vectorize!(N, {
        if valid {
            result
        } else {
            scalar!(f32::NAN)
        }
    })
}

#[allow(unused_braces)]
fn acos_f64<const N: usize>(
    x: Simd<f64, N>,
    sqrt: impl Fn(Simd<f64, N>) -> Simd<f64, N>,
) -> Simd<f64, N> {
    let (valid, upper, z, r, mut result) = vectorize!(N, {
        let valid = x.abs() <= 1.0;
        let a = if valid { x } else { 0.0 };
        let upper = a.abs() >= 0.5;
        let z = if upper { (1.0 - a.abs()) * 0.5 } else { a * a };
        let r = rational_f64(z);
        let result = scalar!(PIO2) - (a - (scalar!(PIO2_TAIL) - a * r));
        (valid, upper, z, r, result)
    });
    if upper.any() {
        let s = sqrt(upper.select(z, Simd::splat(1.0)));
        let (f, c) = split_root_f64(z, s);
        let reduced = vectorize!(N, {
            if x.is_sign_negative() {
                let w = r * s - scalar!(PIO2_TAIL);
                2.0 * (scalar!(PIO2) - (s + w))
            } else {
                let w = r * s + c;
                2.0 * (f + w)
            }
        });
        result = upper.select(reduced, result);
    }
    vectorize!(N, {
        if valid {
            result
        } else {
            scalar!(f64::NAN)
        }
    })
}

macro_rules! make_fns {
    { $($ty:ident, $len:literal, $asin:ident, $acos:ident)* } => {
        $(paste::paste! {
            /// Computes asin(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
            #[no_mangle]
            pub fn [<vapor_asin_ $ty>](x: $ty) -> $ty {
                $asin(x, crate::[<vapor_sqrt_f64x $len>])
            }

            /// Computes acos(x) in radians for each lane, assuming round-to-nearest, ties-to-even.
            #[no_mangle]
            pub fn [<vapor_acos_ $ty>](x: $ty) -> $ty {
                $acos(x, crate::[<vapor_sqrt_ $ty>])
            }
        })*
    }
}

make_fns! {
    f32x2, 2, asin_f32, acos_f32
    f32x4, 4, asin_f32, acos_f32
    f32x8, 8, asin_f32, acos_f32
    f64x2, 2, asin_f64, acos_f64
    f64x4, 4, asin_f64, acos_f64
    f64x8, 8, asin_f64, acos_f64
}
