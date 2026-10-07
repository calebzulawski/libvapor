#![feature(portable_simd)]

use libvapor_tools::{oracle, Case, Width};
use proptest::{
    prelude::*,
    test_runner::{Config, TestRunner},
};

fn input_strategy(width: Width, op: &str) -> BoxedStrategy<[u64; 3]> {
    let all = match width {
        Width::F32 => (proptest::num::f32::ANY | proptest::num::f32::SIGNALING_NAN)
            .prop_map(|x| x.to_bits() as u64)
            .boxed(),
        Width::F64 => (proptest::num::f64::ANY | proptest::num::f64::SIGNALING_NAN)
            .prop_map(f64::to_bits)
            .boxed(),
    };
    // Random sampling rarely hits powers of two or table interval edges.
    let fraction_bits = match width {
        Width::F32 => f32::MANTISSA_DIGITS - 1,
        Width::F64 => f64::MANTISSA_DIGITS - 1,
    };
    let boundaries =
        (any::<u64>(), 0..=fraction_bits, -1_i64..=1).prop_map(move |(bits, clear, offset)| {
            let bits = (bits & !((1_u64 << clear) - 1)).wrapping_add_signed(offset);
            match width {
                Width::F32 => bits as u32 as u64,
                Width::F64 => bits,
            }
        });
    let all = prop_oneof![all, boundaries];
    // Give useful finite inputs as much weight as the general float strategy.
    let (low, high) = match op {
        "asin" | "acos" | "atanh" => (-1.0, 1.0),
        "acosh" => (1.0, 256.0),
        "exp" if width == Width::F32 => (-104.0, 89.0),
        "exp" => (-746.0, 710.0),
        "exp2" if width == Width::F32 => (-151.0, 129.0),
        "exp2" => (-1076.0, 1025.0),
        "log" | "log2" | "log10" => (0.125, 256.0),
        "log1p" => (-1.0, 16.0),
        _ => (-16.0, 16.0),
    };
    let ordinary = (low..high).prop_map(move |x| width.bits(x));
    let value = prop_oneof![all, ordinary];
    if matches!(op, "atan2" | "hypot" | "pow" | "fmod" | "remainder") {
        (value.clone(), value).prop_map(|(y, x)| [y, x, 0]).boxed()
    } else {
        value.prop_map(|x| [x, 0, 0]).boxed()
    }
}

macro_rules! transcendental_tests {
    ($($op:ident $args:tt => $outputs:expr;)+) => {
        paste::paste! {
            $(#[test]
            fn [<live_ $op>]() {
                fn check<const N: usize>(width: Width, cases: &[Case]) {
                    match width {
                        Width::F32 => libvapor_tools::check_accuracy!(f32, F32, N, cases,
                            vapor::[<$op _f32>], $args => $outputs),
                        Width::F64 => libvapor_tools::check_accuracy!(f64, F64, N, cases,
                            vapor::[<$op _f64>], $args => $outputs),
                    }
                }
                fn check_lanes(width: Width, cases: &[Case]) {
                    check::<1>(width, cases);
                    check::<2>(width, cases);
                    check::<3>(width, cases);
                    check::<4>(width, cases);
                    check::<8>(width, cases);
                    check::<16>(width, cases);
                    check::<64>(width, cases);
                }
                for width in [Width::F32, Width::F64] {
                    let lanes = if width == Width::F32 { 8 } else { 4 };
                    let strategy = proptest::collection::vec(input_strategy(width, stringify!($op)), lanes);
                    let mut runner = TestRunner::new(Config::with_source_file(file!()));
                    runner.run(&strategy, |input| {
                        // Calculate each Rug reference once, then check every lane size.
                        let cases: Vec<_> = input.into_iter()
                            .map(|input| oracle::case(width, stringify!($op), input)).collect();
                        check_lanes(width, &cases);
                        Ok(())
                    }).unwrap_or_else(|error| panic!("{width:?} {}: {error}", stringify!($op)));
                }
            })+
        }
    };
}

transcendental_tests! {
    fmod(x, y) => |v| [v];
    remainder(x, y) => |v| [v];
    cbrt(x) => |v| [v];
    hypot(x, y) => |v| [v];
    exp(x) => |v| [v];
    exp2(x) => |v| [v];
    expm1(x) => |v| [v];
    pow(x, y) => |v| [v];
    log(x) => |v| [v];
    log2(x) => |v| [v];
    log10(x) => |v| [v];
    log1p(x) => |v| [v];
    sin(x) => |v| [v];
    cos(x) => |v| [v];
    sincos(x) => |(s, c)| [s, c];
    tan(x) => |v| [v];
    asin(x) => |v| [v];
    acos(x) => |v| [v];
    atan(x) => |v| [v];
    atan2(y, x) => |v| [v];
    sinh(x) => |v| [v];
    cosh(x) => |v| [v];
    tanh(x) => |v| [v];
    asinh(x) => |v| [v];
    acosh(x) => |v| [v];
    atanh(x) => |v| [v];
    erf(x) => |v| [v];
    erfc(x) => |v| [v];
    lgamma(x) => |v| [v];
    tgamma(x) => |v| [v];
}
