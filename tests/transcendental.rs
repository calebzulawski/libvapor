#![feature(portable_simd)]

use libvapor_tools::{serialization::cases, Width};

macro_rules! transcendental_tests {
    ($($op:ident $args:tt => $outputs:expr;)+) => {
        paste::paste! {
            $(#[test]
            fn $op() {
                fn check<const N: usize>() {
                    libvapor_tools::check_accuracy!(f32, F32, N, cases(Width::F32, stringify!($op)),
                        vapor::[<$op _f32>], $args => $outputs);
                    libvapor_tools::check_accuracy!(f64, F64, N, cases(Width::F64, stringify!($op)),
                        vapor::[<$op _f64>], $args => $outputs);
                }
                check::<1>();
                check::<2>();
                check::<3>();
                check::<4>();
                check::<8>();
                check::<16>();
                check::<64>();
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
