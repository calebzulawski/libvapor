//! Operations on adjacent representable floating-point values.

use core::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdFloat,
    Select, Simd,
};

use super::nan::{canonicalize_f32, canonicalize_f64};

macro_rules! adjacent_values {
    ($nextup:ident, $nextdown:ident, $nextafter:ident, $canonicalize:ident, $kind:ident) => {
        /// Returns the next representable value greater than each input.
        /// Positive infinity is unchanged, and NaNs are made quiet.
        #[inline]
        pub fn $nextup<const N: usize>(x: Simd<$kind, N>) -> Simd<$kind, N> {
            let bits = x.to_bits();
            let next = x
                .is_sign_negative()
                .select(bits - Simd::splat(1), bits + Simd::splat(1));
            let next = x.simd_eq(Simd::splat(0.0)).select(Simd::splat(1), next);
            (x.is_nan() | x.simd_eq(Simd::splat($kind::INFINITY)))
                .select($canonicalize(x), Simd::from_bits(next))
        }

        /// Returns the next representable value less than each input.
        /// Negative infinity is unchanged, and NaNs are made quiet.
        #[inline]
        pub fn $nextdown<const N: usize>(x: Simd<$kind, N>) -> Simd<$kind, N> {
            -$nextup(-x)
        }

        /// Steps each x toward y by one representable value; equal inputs return y.
        /// NaNs propagate and are made quiet.
        #[inline]
        pub fn $nextafter<const N: usize>(x: Simd<$kind, N>, y: Simd<$kind, N>) -> Simd<$kind, N> {
            let next = x.simd_lt(y).select($nextup(x), $nextdown(x));
            let next = x.simd_eq(y).select(y, next);
            let nan = x.is_nan().select(x, y);
            (x.is_nan() | y.is_nan()).select($canonicalize(nan), next)
        }
    };
}

adjacent_values!(
    nextup_f32,
    nextdown_f32,
    nextafter_f32,
    canonicalize_f32,
    f32
);
adjacent_values!(
    nextup_f64,
    nextdown_f64,
    nextafter_f64,
    canonicalize_f64,
    f64
);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unit_tests {
        ($kind:ident, $bits:ty) => {
            paste::paste! {
                #[test]
                fn [<adjacent_boundaries_ $kind>]() {
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

                    let tiny = $kind::from_bits(1);
                    let values = [$kind::NEG_INFINITY, -$kind::MAX, -$kind::MIN_POSITIVE, -tiny,
                        -0.0, 0.0, tiny, $kind::MIN_POSITIVE, -1.0, 1.0, $kind::MAX,
                        $kind::INFINITY, SIGNALING, -SIGNALING, $kind::NAN, -$kind::NAN];
                    let x = Simd::from_array(values);
                    check([<nextup_ $kind>](x), values.map($kind::next_up));
                    check([<nextdown_ $kind>](x), values.map($kind::next_down));
                    let x = Simd::from_array([-0.0, 0.0, $kind::INFINITY, $kind::NEG_INFINITY, tiny, -tiny, SIGNALING, 1.0]);
                    let y = Simd::from_array([0.0, -0.0, 0.0, 0.0, -1.0, 1.0, 0.0, SIGNALING]);
                    check([<nextafter_ $kind>](x, y), [0.0, -0.0, $kind::MAX, -$kind::MAX, 0.0, -0.0, $kind::NAN, $kind::NAN]);
                }
            }
        };
    }

    unit_tests!(f32, u32);
    unit_tests!(f64, u64);
}
