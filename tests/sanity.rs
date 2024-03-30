#![feature(portable_simd)]

use core::simd::prelude::*;
use float_eq::assert_float_eq;
use paste::paste;
use proptest as pt;
use vapor::*;

macro_rules! unary_test {
    { $name:ident, $scalar_fn:expr } => {
        unary_test! { $name, $scalar_fn, f32, 2 }
        unary_test! { $name, $scalar_fn, f32, 4 }
        unary_test! { $name, $scalar_fn, f32, 8 }
        unary_test! { $name, $scalar_fn, f64, 2 }
        unary_test! { $name, $scalar_fn, f64, 4 }
        unary_test! { $name, $scalar_fn, f64, 8 }
    };
    { $name:ident, $scalar_fn:expr, $ty:ident, $len:literal } => {
        paste! {
            pt::proptest! {
                #[test]
                fn [<$name _ $ty x $len>](v in pt::array::[<uniform $len>](pt::num::$ty::ANY)) {
                    let got = [<vapor_ $name _ $ty x $len>](Simd::from_array(v)).to_array();
                    for (i, v) in v.iter().copied().enumerate() {
                        let expect = $scalar_fn(v);
                        if got[i].is_nan() && expect.is_nan() {
                            continue
                        } else {
                            assert_float_eq!(got[i], expect, ulps <= 1)
                        }
                    }
                }
            }
        }
    }
}

unary_test! { trunc, num::Float::trunc }
unary_test! { fract, num::Float::fract }
unary_test! { floor, num::Float::floor }
unary_test! { ceil, num::Float::ceil }
unary_test! { round, num::Float::round }
unary_test! { sqrt, num::Float::sqrt }

macro_rules! ternary_test {
    { $name:ident, $scalar_fn:expr } => {
        ternary_test! { $name, $scalar_fn, f32, 2 }
        ternary_test! { $name, $scalar_fn, f32, 4 }
        ternary_test! { $name, $scalar_fn, f32, 8 }
    };
    { $name:ident, $scalar_fn:expr, $ty:ident, $len:literal } => {
        paste! {
            pt::proptest! {
                #[test]
                fn [<$name _ $ty x $len>](
                    a in pt::array::[<uniform $len>](pt::num::$ty::ANY),
                    b in pt::array::[<uniform $len>](pt::num::$ty::ANY),
                    c in pt::array::[<uniform $len>](pt::num::$ty::ANY),
                ) {
                    let got = [<vapor_ $name _ $ty x $len>](
                        Simd::from_array(a),
                        Simd::from_array(b),
                        Simd::from_array(c),
                    ).to_array();
                    for (i, (a, (b, c))) in a.iter().copied().zip(b.iter().copied().zip(c.iter().copied())).enumerate() {
                        let expect = $scalar_fn(a, b, c);
                        if got[i].is_nan() && expect.is_nan() {
                            continue
                        } else {
                            assert_float_eq!(got[i], expect, ulps <= 1)
                        }
                    }
                }
            }
        }
    }
}

ternary_test! { fma, num::Float::mul_add }
