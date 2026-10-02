#![feature(portable_simd, f128)]

use core::simd::prelude::*;
use float_eq::assert_float_eq;
use paste::paste;
use proptest as pt;
use vapor::*;

macro_rules! unary_test {
    { $name:ident, $scalar_fn:expr } => {
        unary_test! { $name, $scalar_fn, $scalar_fn }
    };
    { $name:ident, $scalar_f32:expr, $scalar_f64:expr } => {
        unary_test! { $name, $scalar_f32, f32, 2 }
        unary_test! { $name, $scalar_f32, f32, 4 }
        unary_test! { $name, $scalar_f32, f32, 8 }
        unary_test! { $name, $scalar_f64, f64, 2 }
        unary_test! { $name, $scalar_f64, f64, 4 }
        unary_test! { $name, $scalar_f64, f64, 8 }
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

unary_test! { exp2, num::Float::exp2 }
unary_test! { exp, num::Float::exp }

// Platform log10 implementations can differ by two ULP from the correctly
// rounded result. Round higher-precision references to each lane's type.
unary_test! { log, |v: f32| (v as f128).ln() as f32, |v: f64| (v as f128).ln() as f64 }
unary_test! { log2, |v: f32| (v as f128).log2() as f32, |v: f64| (v as f128).log2() as f64 }
unary_test! { log10, |v: f32| (v as f128).log10() as f32, |v: f64| (v as f128).log10() as f64 }
unary_test! { log1p, |v: f32| (v as f128).ln_1p() as f32, |v: f64| (v as f128).ln_1p() as f64 }

unary_test! { sin, num::Float::sin }
unary_test! { cos, num::Float::cos }
unary_test! { tan, num::Float::tan }
unary_test! { atan, num::Float::atan }
unary_test! { asin, num::Float::asin }
unary_test! { acos, num::Float::acos }

macro_rules! unary_pair_test {
    { $name:ident, $scalar_fn:expr } => {
        unary_pair_test! { $name, $scalar_fn, f32, 2 }
        unary_pair_test! { $name, $scalar_fn, f32, 4 }
        unary_pair_test! { $name, $scalar_fn, f32, 8 }
        unary_pair_test! { $name, $scalar_fn, f64, 2 }
        unary_pair_test! { $name, $scalar_fn, f64, 4 }
        unary_pair_test! { $name, $scalar_fn, f64, 8 }
    };
    { $name:ident, $scalar_fn:expr, $ty:ident, $len:literal } => {
        paste! {
            pt::proptest! {
                #[test]
                fn [<$name _ $ty x $len>](v in pt::array::[<uniform $len>](pt::num::$ty::ANY)) {
                    let (a, b) = [<vapor_ $name _ $ty x $len>](Simd::from_array(v));
                    for (i, v) in v.iter().copied().enumerate() {
                        let (expect_a, expect_b) = $scalar_fn(v);
                        for (got, expect) in [(a[i], expect_a), (b[i], expect_b)] {
                            if got.is_nan() && expect.is_nan() {
                                continue
                            } else {
                                assert_float_eq!(got, expect, ulps <= 1)
                            }
                        }
                    }
                }
            }
        }
    }
}

unary_pair_test! { sincos, num::Float::sin_cos }

macro_rules! binary_test {
    { $name:ident, $scalar_fn:expr } => {
        binary_test! { $name, $scalar_fn, f32, 2 }
        binary_test! { $name, $scalar_fn, f32, 4 }
        binary_test! { $name, $scalar_fn, f32, 8 }
        binary_test! { $name, $scalar_fn, f64, 2 }
        binary_test! { $name, $scalar_fn, f64, 4 }
        binary_test! { $name, $scalar_fn, f64, 8 }
    };
    { $name:ident, $scalar_fn:expr, $ty:ident, $len:literal } => {
        paste! {
            pt::proptest! {
                #[test]
                fn [<$name _ $ty x $len>](
                    a in pt::array::[<uniform $len>](pt::num::$ty::ANY),
                    b in pt::array::[<uniform $len>](pt::num::$ty::ANY),
                ) {
                    let got = [<vapor_ $name _ $ty x $len>](Simd::from_array(a), Simd::from_array(b)).to_array();
                    for (i, (a, b)) in a.iter().copied().zip(b.iter().copied()).enumerate() {
                        let expect = $scalar_fn(a, b);
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

binary_test! { atan2, num::Float::atan2 }

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
