#![feature(portable_simd)]
#![cfg_attr(feature = "lib", no_std)]

use core::simd::{prelude::*, LaneCount, SupportedLaneCount};

#[cfg(feature = "lib")]
mod panic {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_panic: &PanicInfo<'_>) -> ! {
        loop {}
    }
}

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
    { $($ty:ident, $scalar:ty, $unsigned:ty, $signed:ty)* } => {
        $(
        paste::paste! {
            #[no_mangle]
            pub fn [<vapor_fract_ $ty>](x: $ty) -> $ty {
                x - [<vapor_trunc_ $ty>](x)
            }

            #[no_mangle]
            pub fn [<vapor_trunc_ $ty>](x: $ty) -> $ty {
                let b = Simd::splat(if <$ty>::is_f32() { 9 } else { 12 });
                let e = x.exponent() - Simd::splat(<$scalar>::MAX_EXP as $signed - 1) + b;
                e.simd_ge(<$ty>::mantissa_digits() + b).select(x, {
                    let e = e.simd_lt(b).select(Simd::splat(1), e);
                    let m =  Simd::splat(-1i64 as i64 as $unsigned) >> e.cast::<$unsigned>();
                    (x.to_bits() & m).simd_eq(Simd::splat(0)).select(x, Simd::from_bits(x.to_bits() & !m))
                })
            }

            #[no_mangle]
            pub fn [<vapor_floor_ $ty>](x: $ty) -> $ty {
                let e = x.exponent().cast::<$signed>() - Simd::splat(<$scalar>::MAX_EXP as $signed - 1);
                e.simd_ge(<$ty>::mantissa_digits()).select(x, {
                    e.simd_ge(Simd::splat(0)).select({
                        let m = (<$ty>::mantissa_mask() >> e).cast::<$unsigned>();
                        (x.to_bits() & m).simd_eq(Simd::splat(0)).select(
                            x,
                            {
                                let offset = x.is_sign_negative().select(m, Simd::splat(0));
                                <$ty>::from_bits((x.to_bits() + offset) & !m)
                            }
                        )
                    }, {
                        x.is_sign_positive().select(
                            Simd::splat(0.0),
                            x.abs().simd_ne(Simd::splat(0.0)).select(Simd::splat(-1.0), x)
                        )
                    })
                })
            }

            #[no_mangle]
            pub fn [<vapor_ceil_ $ty>](x: $ty) -> $ty {
                let e = x.exponent().cast::<$signed>() - Simd::splat(<$scalar>::MAX_EXP as $signed - 1);
                e.simd_ge(<$ty>::mantissa_digits()).select(x, {
                    e.simd_ge(Simd::splat(0)).select({
                        let m = (<$ty>::mantissa_mask() >> e).cast::<$unsigned>();
                        (x.to_bits() & m).simd_eq(Simd::splat(0)).select(
                            x,
                            {
                                let offset = x.is_sign_positive().select(m, Simd::splat(0));
                                <$ty>::from_bits((x.to_bits() + offset) & !m)
                            }
                        )
                    }, {
                        x.is_sign_negative().select(
                            Simd::splat(-0.0),
                            x.abs().simd_ne(Simd::splat(0.0)).select(Simd::splat(1.0), x)
                        )
                    })
                })
            }

            #[no_mangle]
            pub fn [<vapor_round_ $ty>](x: $ty) -> $ty {
                let e = x.exponent();
                let m = if <$ty>::is_f32() { 0x7f } else { 0x3ff };
                e.simd_ge(Simd::splat(m) + <$ty>::mantissa_digits()).select(x, {
                    let xabs = x.abs();
                    e.simd_lt(Simd::splat(m - 1)).select(Simd::splat(0.0) * x, {
                        let x1pm = <$ty>::from_bits(Simd::splat(if <$ty>::is_f32() { 0x4b000000u64 } else { 0x4330000000000000u64 } as $unsigned));
                        let y = xabs + x1pm - x1pm - xabs;
                        let y = y.simd_gt(Simd::splat(0.5)).select(
                            y + xabs - Simd::splat(1.0),
                            y.simd_le(Simd::splat(-0.5)).select(
                                y + xabs + Simd::splat(1.0),
                                y + xabs,
                            )
                        );
                        x.is_sign_negative().select(-y, y)
                    })
                })
            }
        }
        )*
    }
}

make_fns! {
    f32x2, f32, u32, i32
    f32x4, f32, u32, i32
    f32x8, f32, u32, i32
    f64x2, f64, u64, i64
    f64x4, f64, u64, i64
    f64x8, f64, u64, i64
}
