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
    // Give useful finite inputs as much weight as the full float space.
    let (low, high) = match op {
        "asin" | "acos" => (-1.0, 1.0),
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
    if op == "atan2" {
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
                    let boundaries = log_boundary_cases(width, stringify!($op));
                    if !boundaries.is_empty() {
                        check_lanes(width, &boundaries);
                    }
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
    exp(x) => |v| [v];
    exp2(x) => |v| [v];
    log(x) => |v| [v];
    log2(x) => |v| [v];
    log10(x) => |v| [v];
    log1p(x) => |v| [v];
    sin(x) => |v| [v];
    cos(x) => |v| [v];
    sincos(x) => |(s, c)| [s, c];
    tan(x) => |v| [v];
    atan(x) => |v| [v];
    atan2(y, x) => |v| [v];
    asin(x) => |v| [v];
    acos(x) => |v| [v];
}

fn log_boundary_cases(width: Width, op: &str) -> Vec<Case> {
    if width != Width::F64 || !matches!(op, "log" | "log2" | "log1p") {
        return Vec::new();
    }
    // Sweep adjacent floats at mantissa boundaries, including the reduction
    // interval endpoints, values near one, and the extreme normal exponents.
    let mut input = Vec::new();
    for exponent in [1_u64, 512, 1022, 1023, 1024, 1536, 2046] {
        for fraction in 0..=256_u64 {
            let bits = (exponent << 52) + (fraction << 44);
            input.extend([bits - 1, bits, bits + 1]);
        }
        // The table reduction shifts its interval relative to powers of two.
        for index in 0..=128_u64 {
            let bits = (exponent << 52) + 0x0006900900000000 + (index << 45);
            input.extend([bits - 1, bits, bits + 1]);
        }
    }
    for bit in 0..52 {
        input.extend([(1_u64 << bit) - 1, 1_u64 << bit, (1_u64 << bit) + 1]);
    }
    input.extend([0, 1_u64 << 63, f64::INFINITY.to_bits(), f64::NAN.to_bits()]);
    // Values on both sides of log1p's domain and tiny-input boundary.
    input.extend(
        [
            -1.0_f64,
            (-1.0_f64).next_up(),
            (-1.0_f64).next_down(),
            1.0e-16,
            -1.0e-16,
        ]
        .map(f64::to_bits),
    );

    input
        .into_iter()
        .map(|bits| oracle::case(width, op, [bits, 0, 0]))
        .collect()
}
