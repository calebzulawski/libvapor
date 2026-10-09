//! Basic floating-point operations and predicates.

use core::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdFloat,
    Mask, Simd,
};

macro_rules! operation {
    ($doc:literal, $f32:ident, $f64:ident, ($($arg:ident),+), $body:expr) => {
        #[doc = $doc]
        #[inline]
        pub fn $f32<const N: usize>($($arg: Simd<f32, N>),+) -> Simd<f32, N> {
            $body
        }

        #[doc = $doc]
        #[inline]
        pub fn $f64<const N: usize>($($arg: Simd<f64, N>),+) -> Simd<f64, N> {
            $body
        }
    };
}

operation!(
    "Computes the absolute value of each lane.",
    fabs_f32,
    fabs_f64,
    (x),
    x.abs()
);
operation!(
    "Returns the magnitude of x with the sign of y for each lane.",
    copysign_f32,
    copysign_f64,
    (x, y),
    x.copysign(y)
);
operation!(
    "Returns the smaller value for each lane, returning the other value if one is NaN.",
    fmin_f32,
    fmin_f64,
    (x, y),
    x.simd_min(y)
);
operation!(
    "Returns the larger value for each lane, returning the other value if one is NaN.",
    fmax_f32,
    fmax_f64,
    (x, y),
    x.simd_max(y)
);

macro_rules! predicate {
    ($doc:literal, $f32:ident, $f64:ident, ($($arg:ident),+), $body:expr) => {
        #[doc = $doc]
        #[inline]
        pub fn $f32<const N: usize>($($arg: Simd<f32, N>),+) -> Mask<i32, N> {
            $body
        }

        #[doc = $doc]
        #[inline]
        pub fn $f64<const N: usize>($($arg: Simd<f64, N>),+) -> Mask<i64, N> {
            $body
        }
    };
}

predicate!(
    "Tests whether each lane is finite.",
    isfinite_f32,
    isfinite_f64,
    (x),
    x.is_finite()
);
predicate!(
    "Tests whether each lane is positive or negative infinity.",
    isinf_f32,
    isinf_f64,
    (x),
    x.is_infinite()
);
predicate!(
    "Tests whether each lane is NaN.",
    isnan_f32,
    isnan_f64,
    (x),
    x.is_nan()
);
predicate!(
    "Tests whether each lane is normal, excluding zeros, subnormals, infinities, and NaNs.",
    isnormal_f32,
    isnormal_f64,
    (x),
    x.is_normal()
);
predicate!(
    "Tests whether each lane is subnormal.",
    issubnormal_f32,
    issubnormal_f64,
    (x),
    x.is_subnormal()
);
predicate!(
    "Tests the sign bit of each lane, including zeros and NaNs.",
    signbit_f32,
    signbit_f64,
    (x),
    x.is_sign_negative()
);
predicate!(
    "Tests whether each lane is positive or negative zero.",
    iszero_f32,
    iszero_f64,
    (x),
    x.simd_eq(Simd::splat(0.0))
);
predicate!(
    "Tests whether x is greater than y for each lane; NaNs compare false.",
    isgreater_f32,
    isgreater_f64,
    (x, y),
    x.simd_gt(y)
);
predicate!(
    "Tests whether x is greater than or equal to y for each lane; NaNs compare false.",
    isgreaterequal_f32,
    isgreaterequal_f64,
    (x, y),
    x.simd_ge(y)
);
predicate!(
    "Tests whether x is less than y for each lane; NaNs compare false.",
    isless_f32,
    isless_f64,
    (x, y),
    x.simd_lt(y)
);
predicate!(
    "Tests whether x is less than or equal to y for each lane; NaNs compare false.",
    islessequal_f32,
    islessequal_f64,
    (x, y),
    x.simd_le(y)
);
predicate!(
    "Tests whether x is less than or greater than y for each lane; NaNs compare false.",
    islessgreater_f32,
    islessgreater_f64,
    (x, y),
    x.simd_lt(y) | x.simd_gt(y)
);
predicate!(
    "Tests whether either argument is NaN for each lane.",
    isunordered_f32,
    isunordered_f64,
    (x, y),
    x.is_nan() | y.is_nan()
);
