//! Minimum and maximum operations with explicit NaN and signed-zero behavior.

use core::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdFloat,
    Select, Simd,
};

use super::nan::{canonicalize_f32, canonicalize_f64};
use super::predicates::{totalorder_f32, totalorder_f64};

macro_rules! extremum {
    (
        $doc:literal, $f32:ident, $f64:ident,
        maximum = $maximum:literal, numeric = $numeric:literal, magnitude = $magnitude:literal
    ) => {
        #[doc = $doc]
        #[inline]
        pub fn $f32<const N: usize>(x: Simd<f32, N>, y: Simd<f32, N>) -> Simd<f32, N> {
            extremum_f32::<N, $maximum, $numeric, $magnitude>(x, y)
        }

        #[doc = $doc]
        #[inline]
        pub fn $f64<const N: usize>(x: Simd<f64, N>, y: Simd<f64, N>) -> Simd<f64, N> {
            extremum_f64::<N, $maximum, $numeric, $magnitude>(x, y)
        }
    };
}

macro_rules! extremum_kernel {
    ($function:ident, $kind:ident, $order:ident, $canonicalize:ident) => {
        #[inline]
        fn $function<
            const N: usize,
            const MAXIMUM: bool,
            const NUMERIC: bool,
            const MAGNITUDE: bool,
        >(
            x: Simd<$kind, N>,
            y: Simd<$kind, N>,
        ) -> Simd<$kind, N> {
            // The existing total order already handles differently signed zeros.
            let choose_x = if MAXIMUM { $order(y, x) } else { $order(x, y) };
            let choose_x = if MAGNITUDE {
                let ax = x.abs();
                let ay = y.abs();
                let extreme = if MAXIMUM {
                    ax.simd_gt(ay)
                } else {
                    ax.simd_lt(ay)
                };
                extreme | (ax.simd_eq(ay) & choose_x)
            } else {
                choose_x
            };
            let value = choose_x.select(x, y);
            let value = if NUMERIC {
                x.is_nan().select(y, y.is_nan().select(x, value))
            } else {
                x.is_nan().select(x, y.is_nan().select(y, value))
            };
            $canonicalize(value)
        }
    };
}

extremum_kernel!(extremum_f32, f32, totalorder_f32, canonicalize_f32);
extremum_kernel!(extremum_f64, f64, totalorder_f64, canonicalize_f64);

extremum!(
    "Returns the maximum for each lane, ordering -0 below +0 and propagating NaNs.",
    fmaximum_f32,
    fmaximum_f64,
    maximum = true,
    numeric = false,
    magnitude = false
);
extremum!(
    "Returns the minimum for each lane, ordering -0 below +0 and propagating NaNs.",
    fminimum_f32,
    fminimum_f64,
    maximum = false,
    numeric = false,
    magnitude = false
);
extremum!(
    "Returns the larger-magnitude value; ties choose the maximum, and NaNs propagate.",
    fmaximum_mag_f32,
    fmaximum_mag_f64,
    maximum = true,
    numeric = false,
    magnitude = true
);
extremum!(
    "Returns the smaller-magnitude value; ties choose the minimum, and NaNs propagate.",
    fminimum_mag_f32,
    fminimum_mag_f64,
    maximum = false,
    numeric = false,
    magnitude = true
);
extremum!(
    "Returns the numeric maximum, ordering -0 below +0; ignores a single NaN.",
    fmaximum_num_f32,
    fmaximum_num_f64,
    maximum = true,
    numeric = true,
    magnitude = false
);
extremum!(
    "Returns the numeric minimum, ordering -0 below +0; ignores a single NaN.",
    fminimum_num_f32,
    fminimum_num_f64,
    maximum = false,
    numeric = true,
    magnitude = false
);
extremum!(
    "Returns the numeric larger-magnitude value; ties choose the maximum, and a single NaN is ignored.",
    fmaximum_mag_num_f32,
    fmaximum_mag_num_f64,
    maximum = true,
    numeric = true,
    magnitude = true
);
extremum!(
    "Returns the numeric smaller-magnitude value; ties choose the minimum, and a single NaN is ignored.",
    fminimum_mag_num_f32,
    fminimum_mag_num_f64,
    maximum = false,
    numeric = true,
    magnitude = true
);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unit_tests {
        ($kind:ident, $bits:ty) => {
            paste::paste! {
                #[test]
                fn [<extrema_ties_and_nans_ $kind>]() {
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

                    let nan = $kind::NAN;
                    let x = Simd::from_array([-0.0, 0.0, -2.0, 3.0, nan, 1.0, SIGNALING, nan]);
                    let y = Simd::from_array([0.0, -0.0, 2.0, -4.0, 1.0, SIGNALING, 1.0, SIGNALING]);
                    check([<fminimum_ $kind>](x, y), [-0.0, -0.0, -2.0, -4.0, nan, nan, nan, nan]);
                    check([<fmaximum_ $kind>](x, y), [0.0, 0.0, 2.0, 3.0, nan, nan, nan, nan]);
                    check([<fminimum_mag_ $kind>](x, y), [-0.0, -0.0, -2.0, 3.0, nan, nan, nan, nan]);
                    check([<fmaximum_mag_ $kind>](x, y), [0.0, 0.0, 2.0, -4.0, nan, nan, nan, nan]);
                    check([<fminimum_num_ $kind>](x, y), [-0.0, -0.0, -2.0, -4.0, 1.0, 1.0, 1.0, nan]);
                    check([<fmaximum_num_ $kind>](x, y), [0.0, 0.0, 2.0, 3.0, 1.0, 1.0, 1.0, nan]);
                    check([<fminimum_mag_num_ $kind>](x, y), [-0.0, -0.0, -2.0, 3.0, 1.0, 1.0, 1.0, nan]);
                    check([<fmaximum_mag_num_ $kind>](x, y), [0.0, 0.0, 2.0, -4.0, 1.0, 1.0, 1.0, nan]);
                }
            }
        };
    }

    unit_tests!(f32, u32);
    unit_tests!(f64, u64);
}
