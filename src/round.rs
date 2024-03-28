use core::simd::{prelude::*, LaneCount, SupportedLaneCount};
use simd_macros::vectorize;

trait Float {
    type Int;

    fn is_f32() -> bool;
    fn mantissa_digits() -> Self::Int;
    fn mantissa_mask() -> Self::Int;
    fn exponent(self) -> Self::Int;
}

macro_rules! impl_float {
    { $ty:ty, $int:ty } => {
        impl<const N: usize> Float for Simd<$ty, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            type Int = Simd<$int, N>;

            fn is_f32() -> bool {
                core::mem::size_of::<$int>() == 4
            }

            fn mantissa_digits() -> Self::Int {
                Simd::splat((<$ty>::MANTISSA_DIGITS - 1) as $int)
            }

            fn mantissa_mask() -> Self::Int {
                Simd::splat((1 << (<$ty>::MANTISSA_DIGITS - 1)) - 1)
            }

            fn exponent(self) -> Self::Int {
                self.abs().to_bits().cast() >> Self::mantissa_digits()
            }
        }
    }
}

impl_float! { f32, i32 }
impl_float! { f64, i64 }

macro_rules! make_fns {
    { $($ty:ident, $scalar:ty, $len:literal, $unsigned:ty, $signed:ty)* } => {
        $(
        paste::paste! {
            #[no_mangle]
            pub fn [<vapor_fract_ $ty>](x: $ty) -> $ty {
                x - [<vapor_trunc_ $ty>](x)
            }

            #[no_mangle]
            pub fn [<vapor_trunc_ $ty>](x: $ty) -> $ty {
                vectorize!($len, {
                    let b = scalar!(if <$ty>::is_f32() { 9 } else { 12 });
                    let max_exp = scalar!(<$scalar>::MAX_EXP);
                    let e = x.exponent() - (max_exp as $signed - 1) + b;
                    if e >= <$scalar>::mantissa_digits() + b {
                        x
                    } else {
                        let e = if e < b { 1 } else { e };
                        let m = -1i64 as i64 as $unsigned >> e as $unsigned;
                        if x.to_bits() & m == 0 {
                            x
                        } else {
                            <$scalar>::from_bits(x.to_bits() & !m)
                        }
                    }
                })
            }

            #[no_mangle]
            pub fn [<vapor_floor_ $ty>](x: $ty) -> $ty {
                vectorize!($len, {
                    let e = x.exponent() as $signed - scalar!(<$scalar>::MAX_EXP as $signed - 1);
                    if e >= <$scalar>::mantissa_digits() {
                        x
                    } else if e >= 0{
                        let m = (<$scalar>::mantissa_mask() >> e) as $unsigned;
                        if x.to_bits() & m == 0 {
                            x
                        } else {
                            let offset = if x.is_sign_negative() { m } else { 0 };
                            <$scalar>::from_bits((x.to_bits() + offset) & !m)
                        }
                    } else if x.is_sign_positive() {
                        0.0
                    } else if x.abs() != 0.0 {
                        -1.0
                    } else {
                        x
                    }
                })
            }

            #[no_mangle]
            pub fn [<vapor_ceil_ $ty>](x: $ty) -> $ty {
                vectorize!($len, {
                    let e = x.exponent() as $signed - scalar!(<$scalar>::MAX_EXP as $signed - 1);
                    if e >= <$scalar>::mantissa_digits() {
                        x
                    } else if e >= 0 {
                        let m = (<$scalar>::mantissa_mask() >> e) as $unsigned;
                        if x.to_bits() & m == 0 {
                            x
                        } else {
                            let offset = if x.is_sign_positive() { m } else { 0 };
                            <$scalar>::from_bits((x.to_bits() + offset) & !m)
                        }
                    } else if x.is_sign_negative() {
                        -0.0
                    } else if x.abs() != 0.0 {
                        1.0
                    } else {
                        x
                    }
                })
            }

            #[no_mangle]
            pub fn [<vapor_round_ $ty>](x: $ty) -> $ty {
                vectorize!($len, {
                    let e = x.exponent();
                    let m = scalar!(if <$ty>::is_f32() { 0x7f } else { 0x3ff });
                    if e >= m + <$scalar>::mantissa_digits() {
                        x
                    } else if e < m - 1 {
                        0.0 * x
                    } else {
                        let x1pm = <$scalar>::from_bits(scalar!(if <$ty>::is_f32() { 0x4b000000u64 } else { 0x4330000000000000u64 } as $unsigned));
                        let y = x.abs() + x1pm - x1pm - x.abs();
                        let direction: $scalar = if y > 0.5 {
                            -1.0
                        } else if y <= -0.5 {
                            1.0
                        } else {
                            0.0
                        };
                        (y + x.abs() + direction).copysign(x)
                    }
                })
            }
        }
        )*
    }
}

make_fns! {
    f32x2, f32, 2, u32, i32
    f32x4, f32, 4, u32, i32
    f32x8, f32, 8, u32, i32
    f64x2, f64, 2, u64, i64
    f64x4, f64, 4, u64, i64
    f64x8, f64, 8, u64, i64
}
