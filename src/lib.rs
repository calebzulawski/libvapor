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
    { $($ty:ident, $scalar:ty, $bits_scalar:ty)* } => {
        $(
        paste::paste! {
            #[no_mangle]
            pub fn [<vapor_trunc_ $ty>](x: $ty) -> $ty {
                x.is_normal().select(x - x.cast::<$bits_scalar>().cast::<$scalar>(), x)
            }

            #[no_mangle]
            pub fn [<vapor_fract_ $ty>](x: $ty) -> $ty {
                x - [<vapor_trunc_ $ty>](x)
            }
        }
        )*
    }
}

make_fns! {
    f32x2, f32, i32
    f32x4, f32, i32
    f32x8, f32, i32
    f64x2, f64, i64
    f64x4, f64, i64
    f64x8, f64, i64
}
