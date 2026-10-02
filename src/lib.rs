#![feature(portable_simd)]
#![cfg_attr(feature = "lib", no_std)]

mod round;
pub use round::*;

mod sqrt;
pub use sqrt::*;

mod fma;
pub use fma::*;

mod exp;
pub use exp::*;

mod trig;
pub use trig::*;

mod log;
pub use log::*;

#[cfg(feature = "lib")]
mod panic {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_panic: &PanicInfo<'_>) -> ! {
        loop {}
    }
}
