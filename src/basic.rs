//! Basic floating-point operations and predicates.

mod adjacent;
mod extrema;
mod nan;
mod predicates;

pub use adjacent::*;
pub use extrema::*;
pub use nan::*;
pub use predicates::*;

use core::simd::{cmp::SimdPartialOrd, num::SimdFloat, Select, Simd};

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

operation!(
    "Returns x-y when x > y and positive zero otherwise; propagates NaNs.",
    fdim_f32,
    fdim_f64,
    (x, y),
    (x.is_nan() | y.is_nan()).select(x + y, x.simd_gt(y).select(x - y, Simd::splat(0.0)))
);

macro_rules! modf {
    ($function:ident, $kind:ident, $trunc:path, $canonicalize:ident) => {
        /// Splits each lane into (fractional part, integer part), both with the input's sign.
        /// Infinities have a signed-zero fractional part; NaNs produce two quiet NaNs.
        #[inline]
        pub fn $function<const N: usize>(x: Simd<$kind, N>) -> (Simd<$kind, N>, Simd<$kind, N>) {
            let integer = $trunc($canonicalize(x));
            let fraction = x
                .is_infinite()
                .select(Simd::splat(0.0), x - integer)
                .copysign(x);
            (fraction, integer)
        }
    };
}

modf!(modf_f32, f32, crate::trunc_f32, canonicalize_f32);
modf!(modf_f64, f64, crate::trunc_f64, canonicalize_f64);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unit_tests {
        ($kind:ident, $bits:ty) => {
            paste::paste! {
                #[test]
                fn [<modf_signed_zeros_and_infinities_ $kind>]() {
                    const QUIET: $bits = 1 << ($kind::MANTISSA_DIGITS - 2);
                    const SIGNALING: $kind = $kind::from_bits($kind::INFINITY.to_bits() | 7);

                    fn check<const N: usize>(got: Simd<$kind, N>, expected: [$kind; N]) {
                        for (got, expected) in got.to_array().into_iter().zip(expected) {
                            if expected.is_nan() {
                                assert!(got.is_nan() && got.to_bits() & QUIET != 0);
                            } else {
                                assert_eq!(got.to_bits(), expected.to_bits(), "got {got:?}, expected {expected:?}");
                            }
                        }
                    }

                    let x = Simd::from_array([-0.0, 0.0, -3.5, 3.5, -$kind::MAX, $kind::MAX, $kind::NEG_INFINITY, $kind::INFINITY]);
                    let (fraction, integer) = [<modf_ $kind>](x);
                    check(fraction, [-0.0, 0.0, -0.5, 0.5, -0.0, 0.0, -0.0, 0.0]);
                    check(integer, [-0.0, 0.0, -3.0, 3.0, -$kind::MAX, $kind::MAX, $kind::NEG_INFINITY, $kind::INFINITY]);
                    let (fraction, integer) = [<modf_ $kind>](Simd::from_array([SIGNALING, -SIGNALING]));
                    check(fraction, [$kind::NAN; 2]);
                    check(integer, [$kind::NAN; 2]);
                }
            }
        };
    }

    unit_tests!(f32, u32);
    unit_tests!(f64, u64);
}
