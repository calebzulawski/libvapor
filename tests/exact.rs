#![feature(portable_simd)]

use proptest::{
    prelude::*,
    test_runner::{Config, TestRunner},
};
use std::simd::Simd;
use vapor::*;

#[path = "../src/fma/software.rs"]
mod fma_software;
#[path = "../src/sqrt/software.rs"]
mod sqrt_software;
use fma_software::fma_f32 as fma_software_f32;
use sqrt_software::{sqrt_f32 as sqrt_software_f32, sqrt_f64 as sqrt_software_f64};

macro_rules! exact_tests {
    ($kind:ident; $($function:ident ($($arg:ident),+) => $expected:expr;)+) => {
        paste::paste! {
            $(#[test]
            fn [<test_ $function>]() {
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
                        let got = $function($($arg),+).to_array();
                        for lane in 0..N {
                            let [$($arg),+] = input[lane];
                            let expected = $expected;
                            prop_assert!(
                                if expected.is_nan() { got[lane].is_nan() }
                                else { got[lane].to_bits() == expected.to_bits() },
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
}

exact_tests!(f32;
    trunc_f32(x) => x.trunc();
    fract_f32(x) => x.fract();
    floor_f32(x) => x.floor();
    ceil_f32(x) => x.ceil();
    round_f32(x) => x.round();
    sqrt_f32(x) => x.sqrt();
    sqrt_software_f32(x) => x.sqrt();
    fma_f32(x, y, z) => x.mul_add(y, z);
    fma_software_f32(x, y, z) => x.mul_add(y, z);
);
exact_tests!(f64;
    trunc_f64(x) => x.trunc();
    fract_f64(x) => x.fract();
    floor_f64(x) => x.floor();
    ceil_f64(x) => x.ceil();
    round_f64(x) => x.round();
    sqrt_f64(x) => x.sqrt();
    sqrt_software_f64(x) => x.sqrt();
);
