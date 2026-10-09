//! NaN canonicalization and payload operations.

use core::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::{SimdFloat, SimdUint},
    Mask, Select, Simd,
};

macro_rules! nan_payload {
    ($canonicalize:ident, $getpayload:ident, $setpayload:ident, $setpayloadsig:ident, $kind:ident, $bits:ty, $mask:ty) => {
        /// Returns each input unchanged, except signaling NaNs are made quiet.
        /// NaN signs and payloads are preserved.
        #[inline]
        pub fn $canonicalize<const N: usize>(x: Simd<$kind, N>) -> Simd<$kind, N> {
            let quiet = Simd::splat((1 as $bits) << ($kind::MANTISSA_DIGITS - 2));
            Simd::from_bits(x.is_nan().select(x.to_bits() | quiet, x.to_bits()))
        }

        /// Extracts the NaN payload as a nonnegative floating-point integer.
        /// Non-NaN inputs return -1; the sign and quiet bit are not part of the payload.
        #[inline]
        pub fn $getpayload<const N: usize>(x: Simd<$kind, N>) -> Simd<$kind, N> {
            let payload = ((1 as $bits) << ($kind::MANTISSA_DIGITS - 2)) - 1;
            x.is_nan().select(
                (x.to_bits() & Simd::splat(payload)).cast(),
                Simd::splat(-1.0),
            )
        }

        /// Creates positive quiet NaNs from integer payloads in [0, 2^(p-2)).
        /// Here p is `MANTISSA_DIGITS`. Returns the values and a success mask;
        /// invalid lanes return positive zero and false.
        #[inline]
        pub fn $setpayload<const N: usize>(
            payload: Simd<$kind, N>,
        ) -> (Simd<$kind, N>, Mask<$mask, N>) {
            let quiet = (1 as $bits) << ($kind::MANTISSA_DIGITS - 2);
            let integer = payload.cast::<$bits>();
            let valid = payload.simd_ge(Simd::splat(0.0))
                & payload.simd_lt(Simd::splat(quiet as $kind))
                & integer.cast::<$kind>().simd_eq(payload);
            let nan = Simd::from_bits(integer | Simd::splat($kind::INFINITY.to_bits() | quiet));
            (valid.select(nan, Simd::splat(0.0)), valid)
        }

        /// Creates positive signaling NaNs from integer payloads in [1, 2^(p-2)).
        /// Here p is `MANTISSA_DIGITS`. Returns the values and a success mask;
        /// invalid lanes return positive zero and false.
        #[inline]
        pub fn $setpayloadsig<const N: usize>(
            payload: Simd<$kind, N>,
        ) -> (Simd<$kind, N>, Mask<$mask, N>) {
            let limit = (1 as $bits) << ($kind::MANTISSA_DIGITS - 2);
            let integer = payload.cast::<$bits>();
            let valid = payload.simd_gt(Simd::splat(0.0))
                & payload.simd_lt(Simd::splat(limit as $kind))
                & integer.cast::<$kind>().simd_eq(payload);
            let nan = Simd::from_bits(integer | Simd::splat($kind::INFINITY.to_bits()));
            (valid.select(nan, Simd::splat(0.0)), valid)
        }
    };
}

nan_payload!(
    canonicalize_f32,
    getpayload_f32,
    setpayload_f32,
    setpayloadsig_f32,
    f32,
    u32,
    i32
);
nan_payload!(
    canonicalize_f64,
    getpayload_f64,
    setpayload_f64,
    setpayloadsig_f64,
    f64,
    u64,
    i64
);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unit_tests {
        ($kind:ident, $bits:ty) => {
            paste::paste! {
                #[test]
                fn [<payload_limits_and_canonicalization_ $kind>]() {
                    const QUIET: $bits = 1 << ($kind::MANTISSA_DIGITS - 2);
                    const SIGNALING: $kind = $kind::from_bits($kind::INFINITY.to_bits() | 7);

                    let max = (QUIET - 1) as $kind;
                    let payload = Simd::from_array([-0.0, 0.0, 1.0, 7.0, max, -1.0, 0.5, QUIET as $kind]);
                    let (quiet, valid) = [<setpayload_ $kind>](payload);
                    assert_eq!(valid.to_array(), [true, true, true, true, true, false, false, false]);
                    let base = $kind::INFINITY.to_bits() | QUIET;
                    assert_eq!(quiet.to_bits().to_array(), [base, base, base | 1, base | 7, base | (QUIET - 1), 0, 0, 0]);
                    assert_eq!([<getpayload_ $kind>](quiet).to_bits().to_array(),
                        [0.0, 0.0, 1.0, 7.0, max, -1.0, -1.0, -1.0].map($kind::to_bits));

                    let (signaling, valid) = [<setpayloadsig_ $kind>](payload);
                    assert_eq!(valid.to_array(), [false, false, true, true, true, false, false, false]);
                    let base = $kind::INFINITY.to_bits();
                    assert_eq!(signaling.to_bits().to_array(), [0, 0, base | 1, base | 7, base | (QUIET - 1), 0, 0, 0]);
                    let invalid = Simd::from_array([$kind::INFINITY, $kind::NEG_INFINITY, $kind::NAN, $kind::from_bits(1)]);
                    for (value, valid) in [[<setpayload_ $kind>](invalid), [<setpayloadsig_ $kind>](invalid)] {
                        assert!(!valid.any());
                        assert_eq!(value.to_bits().to_array(), [0; 4]);
                    }

                    let x = Simd::from_array([SIGNALING, -SIGNALING, $kind::NAN, -0.0]);
                    assert_eq!([<canonicalize_ $kind>](x).to_bits().to_array(), [
                        SIGNALING.to_bits() | QUIET, (-SIGNALING).to_bits() | QUIET,
                        $kind::NAN.to_bits(), (-0.0 as $kind).to_bits(),
                    ]);
                }
            }
        };
    }

    unit_tests!(f32, u32);
    unit_tests!(f64, u64);
}
