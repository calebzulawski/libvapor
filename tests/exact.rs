#![feature(portable_simd)]

use mwise::*;
use proptest::{
    prelude::*,
    test_runner::{Config, TestRunner},
};
use std::simd::Simd;

macro_rules! exact_tests {
    ($($function:ident ($($arg:ident),+) => $expected:expr;)+) => {
        exact_tests!(@kind f32, float; $($function($($arg),+) => $expected;)+);
        exact_tests!(@kind f64, float; $($function($($arg),+) => $expected;)+);
    };
    (@integer $($function:ident ($($arg:ident),+) => $expected:expr;)+) => {
        exact_tests!(@kind f32, integer; $($function($($arg),+) => $expected;)+);
        exact_tests!(@kind f64, integer; $($function($($arg),+) => $expected;)+);
    };
    (@kind $kind:ident, $result:ident; $($function:ident ($($arg:ident),+) => $expected:expr;)+) => {
        paste::paste! {
            $(#[test]
            fn [<test_ $function _ $kind>]() {
                fn run<const N: usize>() {
                    let value = prop_oneof![
                        3 => proptest::num::$kind::ANY | proptest::num::$kind::SIGNALING_NAN,
                        1 => -16.0 as $kind..16.0 as $kind,
                    ];
                    let strategy = proptest::collection::vec(proptest::array::uniform(value), N);
                    let mut runner = TestRunner::new(Config::with_source_file(file!()));
                    runner.run(&strategy, |input| {
                        let [$($arg),+] = std::array::from_fn(|column| {
                            Simd::<$kind, N>::from_array(std::array::from_fn(|lane| input[lane][column]))
                        });
                        let got = [<$function _ $kind>]($($arg),+).to_array();
                        for lane in 0..N {
                            let [$($arg),+] = input[lane];
                            let expected = $expected;
                            prop_assert!(
                                exact_tests!(@equal $result; got[lane], expected),
                                "{}x{} input {:x?}: got {:?}, expected {:?}",
                                stringify!($kind), N, input[lane].map($kind::to_bits), got[lane], expected
                            );
                        }
                        Ok(())
                    }).unwrap();
                }
                run::<1>();
                run::<2>();
                run::<3>();
                run::<4>();
                run::<8>();
                run::<16>();
                run::<64>();
            })+
        }
    };
    (@equal float; $got:expr, $expected:expr) => {
        if $expected.is_nan() { $got.is_nan() }
        else { $got.to_bits() == $expected.to_bits() }
    };
    (@equal integer; $got:expr, $expected:expr) => {
        $got == $expected
    };
}

exact_tests! {
    trunc(x) => x.trunc();
    floor(x) => x.floor();
    ceil(x) => x.ceil();
    round(x) => x.round();
    roundeven(x) => x.round_ties_even();
    sqrt(x) => x.sqrt();
    fma(x, y, z) => x.mul_add(y, z);
}

exact_tests! { @integer
    lround(x) => x.round() as i32;
    llround(x) => x.round() as i64;
}
