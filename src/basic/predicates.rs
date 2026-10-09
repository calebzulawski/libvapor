//! Floating-point classification, comparisons, and total ordering.

use core::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::{SimdFloat, SimdUint},
    Mask, Select, Simd,
};

/// NaN category returned by the `fpclassify` functions.
pub const FP_NAN: i32 = 0;
/// Infinity category returned by the `fpclassify` functions.
pub const FP_INFINITE: i32 = 1;
/// Zero category returned by the `fpclassify` functions.
pub const FP_ZERO: i32 = 2;
/// Subnormal category returned by the `fpclassify` functions.
pub const FP_SUBNORMAL: i32 = 3;
/// Normal category returned by the `fpclassify` functions.
pub const FP_NORMAL: i32 = 4;

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

macro_rules! classification {
    ($classify:ident, $signaling:ident, $kind:ident, $mask:ty) => {
        /// Returns an `FP_*` category code for each lane with the input's element width.
        ///
        /// The codes are [`FP_NAN`], [`FP_INFINITE`], [`FP_ZERO`],
        /// [`FP_SUBNORMAL`], and [`FP_NORMAL`].
        /// Use `Simd::splat(FP_NORMAL as _)` to construct a comparison vector.
        #[inline]
        pub fn $classify<const N: usize>(x: Simd<$kind, N>) -> Simd<$mask, N> {
            let category = x.simd_eq(Simd::splat(0.0)).select(
                Simd::splat(FP_ZERO as $mask),
                Simd::splat(FP_NORMAL as $mask),
            );
            let category = x
                .is_subnormal()
                .select(Simd::splat(FP_SUBNORMAL as $mask), category);
            let category = x
                .is_infinite()
                .select(Simd::splat(FP_INFINITE as $mask), category);
            x.is_nan().select(Simd::splat(FP_NAN as $mask), category)
        }

        /// Tests whether each lane is a signaling NaN by inspecting its bits.
        #[inline]
        pub fn $signaling<const N: usize>(x: Simd<$kind, N>) -> Mask<$mask, N> {
            let bits = x.to_bits();
            let exponent = $kind::INFINITY.to_bits();
            // MANTISSA_DIGITS includes the implicit leading bit.
            let mantissa = (1 << ($kind::MANTISSA_DIGITS - 1)) - 1;
            // The highest stored mantissa bit identifies quiet NaNs.
            let quiet = (mantissa + 1) >> 1;
            let signaling = (bits & Simd::splat(exponent | quiet)).simd_eq(Simd::splat(exponent));
            let payload = (bits & Simd::splat(mantissa)).simd_ne(Simd::splat(0));
            signaling & payload
        }
    };
}

classification!(fpclassify_f32, issignaling_f32, f32, i32);
classification!(fpclassify_f64, issignaling_f64, f64, i64);

macro_rules! total_order {
    ($order:ident, $magnitude:ident, $kind:ident, $mask:ty) => {
        /// Tests whether x precedes or equals y in IEEE 754 total order for each lane.
        /// Includes signed zeros and NaN signs, signaling bits, and payloads.
        #[inline]
        pub fn $order<const N: usize>(x: Simd<$kind, N>, y: Simd<$kind, N>) -> Mask<$mask, N> {
            // TODO: Use SIMD total_cmp once portable SIMD provides it.
            let x = x.to_bits().cast::<$mask>();
            let y = y.to_bits().cast::<$mask>();
            let magnitude = Simd::splat(<$mask>::MAX);
            let sign_shift = (<$mask>::BITS - 1) as $mask;
            // Reverse the magnitude bits of negative values for signed integer ordering.
            let x = x ^ ((x >> sign_shift) & magnitude);
            let y = y ^ ((y >> sign_shift) & magnitude);
            x.simd_le(y)
        }

        /// Tests whether the magnitude of x precedes or equals that of y in IEEE 754 total order.
        /// Ignores signs, including those of zeros and NaNs.
        #[inline]
        pub fn $magnitude<const N: usize>(x: Simd<$kind, N>, y: Simd<$kind, N>) -> Mask<$mask, N> {
            let x = x.abs().to_bits().cast::<$mask>();
            let y = y.abs().to_bits().cast::<$mask>();
            x.simd_le(y)
        }
    };
}

total_order!(totalorder_f32, totalordermag_f32, f32, i32);
total_order!(totalorder_f64, totalordermag_f64, f64, i64);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unit_tests {
        ($kind:ident, $bits:ty) => {
            paste::paste! {
                #[test]
                fn [<total_order_zeros_and_nan_encodings_ $kind>]() {
                    const QUIET: $bits = 1 << ($kind::MANTISSA_DIGITS - 2);
                    const SIGNALING: $kind = $kind::from_bits($kind::INFINITY.to_bits() | 7);

                    let nan = $kind::from_bits(SIGNALING.to_bits() | QUIET);
                    let small = $kind::from_bits($kind::INFINITY.to_bits() | 1);
                    let x = [-nan, -SIGNALING, -0.0, 0.0, SIGNALING, nan, small, $kind::from_bits(small.to_bits() | QUIET)];
                    let y = [-SIGNALING, -nan, 0.0, -0.0, nan, SIGNALING, SIGNALING, nan];
                    let expected = std::array::from_fn(|lane| !x[lane].total_cmp(&y[lane]).is_gt());
                    assert_eq!([<totalorder_ $kind>](Simd::from_array(x), Simd::from_array(y)).to_array(), expected);
                    let expected = std::array::from_fn(|lane| !x[lane].abs().total_cmp(&y[lane].abs()).is_gt());
                    assert_eq!([<totalordermag_ $kind>](Simd::from_array(x), Simd::from_array(y)).to_array(), expected);
                    assert!([<totalorder_ $kind>](Simd::from_array(x), Simd::from_array(x)).all());
                }
            }
        };
    }

    unit_tests!(f32, u32);
    unit_tests!(f64, u64);
}
