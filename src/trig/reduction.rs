#[path = "reduction/f32.rs"]
mod f32;
#[path = "reduction/f64.rs"]
mod f64;
#[path = "reduction/large_f32.rs"]
mod large_f32;
#[path = "reduction/large_f64.rs"]
mod large_f64;
#[path = "reduction/medium_f64.rs"]
mod medium_f64;
#[path = "reduction/quadrant.rs"]
mod quadrant;

pub(crate) use f32::reduce_f32;
pub(crate) use f64::reduce_f64;

use core::simd::prelude::*;
use quadrant::round_quadrant;

pub(crate) struct Reduced<const N: usize> {
    pub(super) quadrant: Simd<u64, N>,
    pub(super) hi: Simd<f64, N>,
    pub(super) lo: Simd<f64, N>,
}
