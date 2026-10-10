#![feature(portable_simd, test, float_gamma, float_erf)]

extern crate test;

use std::{hint::black_box, simd::Simd};
use test::Bencher;

mod libm {
    unsafe extern "C" {
        pub(super) fn remainder(x: f64, y: f64) -> f64;
        pub(super) fn remainderf(x: f32, y: f32) -> f32;
        pub(super) fn remquo(x: f64, y: f64, quotient: *mut i32) -> f64;
        pub(super) fn remquof(x: f32, y: f32, quotient: *mut i32) -> f32;
    }
}

// One iteration processes 32 vectors. Input generation is outside the timer.
const BATCH_SIZE: usize = 32;

// Match input/output alignment across the Rust and mwise paths.
#[repr(align(64))]
struct Aligned<T>(T);

fn inputs<const N: usize>(min: f64, max: f64, seed: u64) -> [[f64; N]; BATCH_SIZE] {
    let mut state = seed;
    std::array::from_fn(|_| {
        std::array::from_fn(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let unit = (state >> 11) as f64 / (1_u64 << 53) as f64;
            min + (max - min) * unit
        })
    })
}

// The same loop handles unary, binary, ternary, and paired-output operations.
macro_rules! benchmark {
    ($name:ident, ($($arg:ident $( : $arg_type:ty)?),+), $scalar:ident, $min:expr, $max:expr) => {
        benchmark!($name, ($($arg $( : $arg_type)?),+), $scalar, $min, $max, -16.0, 16.0);
    };
    ($name:ident, ($($arg:ident $( : $arg_type:ty)?),+), $scalar:ident, $min:expr, $max:expr, $nmin:expr, $nmax:expr) => {
        benchmark!(@kind $name, ($($arg $( : $arg_type)?),+), $scalar, $min, $max, $nmin, $nmax, f32, 16);
        benchmark!(@kind $name, ($($arg $( : $arg_type)?),+), $scalar, $min, $max, $nmin, $nmax, f64, 8);
    };
    (@kind $name:ident, ($($arg:ident $( : $arg_type:ty)?),+), $scalar:ident, $min:expr, $max:expr, $nmin:expr, $nmax:expr, $kind:ident, $lanes:literal) => {
        paste::paste! {
            mod [<$name _ $kind x $lanes>] {
                use super::*;

                #[bench]
                fn scalar(b: &mut Bencher) {
                    let mut columns = 0..;
                    $(let $arg = {
                        let column = columns.next().unwrap();
                        benchmark!(@inputs $kind, $lanes, $min, $max, $nmin, $nmax, column $(, $arg_type)?)
                    };)+
                    b.iter(|| {
                        for batch in 0..BATCH_SIZE {
                            $(let $arg = black_box(&$arg[batch]);)+
                            benchmark!(@scalar_output $name, $kind, $lanes, $scalar, ($($arg),+));
                        }
                    });
                }

                #[bench]
                fn vector(b: &mut Bencher) {
                    let mut columns = 0..;
                    $(let $arg = {
                        let column = columns.next().unwrap();
                        benchmark!(@inputs $kind, $lanes, $min, $max, $nmin, $nmax, column $(, $arg_type)?)
                    };)+
                    b.iter(|| {
                        for batch in 0..BATCH_SIZE {
                            $(let $arg = black_box(&$arg[batch]);)+
                            benchmark!(@output $name, mwise::[<$name _ $kind>]($(*$arg),+));
                        }
                    });
                }
            }
        }
    };
    (@inputs $kind:ident, $lanes:literal, $min:expr, $max:expr, $nmin:expr, $nmax:expr, $column:ident) => {
        inputs::<$lanes>($min, $max, $column as u64 + 1)
            .map(|batch| Simd::from_array(batch.map(|x| x as $kind)))
    };
    (@inputs $kind:ident, $lanes:literal, $min:expr, $max:expr, $nmin:expr, $nmax:expr, $column:ident, $arg_type:ty) => {
        inputs::<$lanes>($nmin, $nmax, $column as u64 + 1)
            .map(|batch| Simd::from_array(batch.map(|x| x as $arg_type)))
    };
    (@scalar_output rsqrt, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            1.0 / $x[lane].sqrt()
        })))
    };
    (@scalar_output pown, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident, $n:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            // The benchmark's integer range fits the scalar powi API.
            $x[lane].powi($n[lane] as i32)
        })))
    };
    (@scalar_output rootn, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident, $n:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            if $n[lane] == 0 { $kind::NAN }
            else { $x[lane].powf(1.0 / $n[lane] as $kind) }
        })))
    };
    (@scalar_output compoundn, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident, $n:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            (1.0 + $x[lane]).powi($n[lane] as i32)
        })))
    };
    (@scalar_output fmod, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident, $y:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            $x[lane] % $y[lane]
        })))
    };
    (@scalar_output remainder, f32, $lanes:literal, $scalar:ident, ($x:ident, $y:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| unsafe {
            libm::remainderf($x[lane], $y[lane])
        })))
    };
    (@scalar_output remainder, f64, $lanes:literal, $scalar:ident, ($x:ident, $y:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| unsafe {
            libm::remainder($x[lane], $y[lane])
        })))
    };
    (@scalar_output remquo, f32, $lanes:literal, $scalar:ident, ($x:ident, $y:ident)) => {
        benchmark!(@remquo_scalar $lanes, libm::remquof, $x, $y)
    };
    (@scalar_output remquo, f64, $lanes:literal, $scalar:ident, ($x:ident, $y:ident)) => {
        benchmark!(@remquo_scalar $lanes, libm::remquo, $x, $y)
    };
    (@remquo_scalar $lanes:literal, $function:path, $x:ident, $y:ident) => {{
        let mut remainder = [0.0; $lanes];
        let mut quotient = [0; $lanes];
        for lane in 0..$lanes {
            remainder[lane] = unsafe { $function($x[lane], $y[lane], &mut quotient[lane]) };
        }
        black_box(Aligned((remainder, quotient)));
    }};
    (@scalar_output lround, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            $x[lane].round() as i32
        })))
    };
    (@scalar_output llround, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            $x[lane].round() as i64
        })))
    };
    (@scalar_output lgamma, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            $kind::$scalar($x[lane]).0
        })))
    };
    (@scalar_output sincos, $kind:ident, $lanes:literal, $scalar:ident, ($x:ident)) => {{
        let mut sin = [0.0; $lanes];
        let mut cos = [0.0; $lanes];
        for lane in 0..$lanes {
            (sin[lane], cos[lane]) = $kind::$scalar($x[lane]);
        }
        black_box(Aligned((sin, cos)));
    }};
    (@scalar_output $name:ident, $kind:ident, $lanes:literal, $scalar:ident, ($($arg:ident),+)) => {
        black_box(Aligned(std::array::from_fn::<_, $lanes, _>(|lane| {
            $kind::$scalar($($arg[lane]),+)
        })))
    };
    (@output sincos, $value:expr) => {{
        let (sin, cos) = $value;
        black_box(Aligned((sin.to_array(), cos.to_array())));
    }};
    (@output remquo, $value:expr) => {{
        let (remainder, quotient) = $value;
        black_box(Aligned((remainder.to_array(), quotient.to_array())));
    }};
    (@output $name:ident, $value:expr) => {
        black_box(Aligned($value.to_array()))
    };
}

// We don't benchmark basic functions that reduce to trivial core::simd operations.
benchmark!(trunc, (x), trunc, -16.0, 16.0);
benchmark!(floor, (x), floor, -16.0, 16.0);
benchmark!(ceil, (x), ceil, -16.0, 16.0);
benchmark!(round, (x), round, -16.0, 16.0);
benchmark!(roundeven, (x), round_ties_even, -16.0, 16.0);
benchmark!(lround, (x), round, -16.0, 16.0);
benchmark!(llround, (x), round, -16.0, 16.0);
benchmark!(fmod, (x, y), fmod, -16.0, 16.0);
benchmark!(remainder, (x, y), remainder, -16.0, 16.0);
benchmark!(remquo, (x, y), remquo, -16.0, 16.0);
benchmark!(sqrt, (x), sqrt, 0.125, 256.0);
benchmark!(rsqrt, (x), sqrt, 0.125, 256.0);
benchmark!(cbrt, (x), cbrt, -16.0, 16.0);
benchmark!(hypot, (x, y), hypot, -16.0, 16.0);
benchmark!(fma, (x, y, z), mul_add, -16.0, 16.0);
benchmark!(exp, (x), exp, -10.0, 10.0);
benchmark!(exp2, (x), exp2, -10.0, 10.0);
benchmark!(expm1, (x), exp_m1, -10.0, 10.0);
benchmark!(pow, (x, y), powf, 0.125, 4.0);
benchmark!(powr, (x, y), powf, 0.125, 4.0);
benchmark!(pown, (x, n: i64), powi, -4.0, 4.0, -128.0, 129.0);
benchmark!(rootn, (x, n: i64), powf, 0.125, 256.0);
benchmark!(compoundn, (x, n: i64), powi, -0.875, 4.0, -128.0, 129.0);
benchmark!(log, (x), ln, 0.125, 256.0);
benchmark!(log2, (x), log2, 0.125, 256.0);
benchmark!(log10, (x), log10, 0.125, 256.0);
benchmark!(log1p, (x), ln_1p, -0.875, 16.0);
benchmark!(sin, (x), sin, -16.0, 16.0);
benchmark!(cos, (x), cos, -16.0, 16.0);
benchmark!(sincos, (x), sin_cos, -16.0, 16.0);
benchmark!(tan, (x), tan, -16.0, 16.0);
benchmark!(asin, (x), asin, -1.0, 1.0);
benchmark!(acos, (x), acos, -1.0, 1.0);
benchmark!(atan, (x), atan, -16.0, 16.0);
benchmark!(atan2, (y, x), atan2, -16.0, 16.0);
benchmark!(sinh, (x), sinh, -16.0, 16.0);
benchmark!(cosh, (x), cosh, -16.0, 16.0);
benchmark!(tanh, (x), tanh, -16.0, 16.0);
benchmark!(asinh, (x), asinh, -16.0, 16.0);
benchmark!(acosh, (x), acosh, 1.0, 16.0);
benchmark!(atanh, (x), atanh, -0.99, 0.99);
benchmark!(erf, (x), erf, -4.0, 4.0);
benchmark!(erfc, (x), erfc, -4.0, 8.0);
benchmark!(lgamma, (x), ln_gamma, -16.0, 16.0);
benchmark!(tgamma, (x), gamma, -16.0, 16.0);
