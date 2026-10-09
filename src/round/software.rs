#[path = "software/f32.rs"]
mod f32;
#[path = "software/f64.rs"]
mod f64;
#[cfg(test)]
#[path = "software/tests.rs"]
mod tests;

pub(crate) use f32::*;
pub(crate) use f64::*;

/// Round toward zero.
const MODE_TRUNC: u8 = 0;
/// Round toward negative infinity.
const MODE_FLOOR: u8 = 1;
/// Round toward positive infinity.
const MODE_CEIL: u8 = 2;
/// Round to nearest, with ties away from zero.
const MODE_ROUND_TIES_AWAY: u8 = 3;
/// Round to nearest, with ties to even.
const MODE_ROUND_TIES_EVEN: u8 = 4;

// Use portable SIMD integer conversions on x86 targets without SSE4.1.
const USE_INT_CONVERSION: bool = cfg!(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "sse4.1")
));
