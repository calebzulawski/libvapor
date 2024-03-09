#![feature(portable_simd)]
#![cfg_attr(feature = "lib", no_std)]

use core::simd::prelude::*;

#[cfg(feature = "lib")]
mod panic {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_panic: &PanicInfo<'_>) -> ! {
        loop {}
    }
}

macro_rules! make_fns {
    { $($ty:ident, $scalar:ty, $unsigned:ty, $signed:ty)* } => {
        $(
        paste::paste! {
            #[no_mangle]
            pub fn [<vapor_fract_ $ty>](x: $ty) -> $ty {
                x.is_normal().select(x - x.cast::<$unsigned>().cast::<$scalar>(), x)
            }

            #[no_mangle]
            pub fn [<vapor_trunc_ $ty>](x: $ty) -> $ty {
                let mplaces = (<$scalar>::MANTISSA_DIGITS - 1) as $signed;
                let b = if core::mem::size_of::<$scalar>() == 4 { 9 } else { 12 };
                let e = (x.abs().to_bits() >> mplaces as $unsigned).cast::<$signed>() - Simd::splat(<$scalar>::MAX_EXP as $signed - 1) + Simd::splat(b);
                e.simd_ge(Simd::splat(mplaces + b)).select(x, {
                    let e = e.simd_lt(Simd::splat(b)).select(Simd::splat(1), e);
                    let m =  Simd::splat(-1i64 as i64 as $unsigned) >> e.cast::<$unsigned>();
                    (x.to_bits() & m).simd_eq(Simd::splat(0)).select(x, Simd::from_bits(x.to_bits() & !m))
                })
            }

            #[no_mangle]
            pub fn [<vapor_floor_ $ty>](x: $ty) -> $ty {
                let trunc = x - [<vapor_trunc_ $ty>](x);
                let floor = x.simd_lt(Simd::splat(0.0)).select(trunc + Simd::splat(1.0), trunc);
                x.is_infinite().select(x, floor)
            }

            #[no_mangle]
            pub fn [<vapor_ceil_ $ty>](x: $ty) -> $ty {
                let trunc = x - [<vapor_trunc_ $ty>](x);
                let ceil = x.simd_le(Simd::splat(0.0)).select(trunc, trunc - Simd::splat(1.0));
                x.is_infinite().select(x, ceil)
            }

            #[no_mangle]
            pub fn [<vapor_round_ $ty>](x: $ty) -> $ty {
                x.is_normal().select(x.cast::<$unsigned>().cast::<$scalar>(), x)
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
