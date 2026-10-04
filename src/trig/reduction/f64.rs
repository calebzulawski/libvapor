// Local SIMD routing between the medium and large f64 reducers.

use super::large_f64::reduce_large_f64;
use super::medium_f64::reduce_medium_f64;
use super::Reduced;

use core::simd::prelude::*;

use super::super::data::*;

pub(crate) fn reduce_f64<const N: usize>(ax: Simd<f64, N>) -> Reduced<N> {
    let large = ax.simd_ge(Simd::splat(REDUCTION_LIMIT_F64));
    let medium = reduce_medium_f64(large.select(Simd::splat(0.0), ax));
    if !large.any() {
        return medium;
    }
    // Exponent extraction and table indices require finite normal inputs.
    let reduced = reduce_large_f64(large.select(ax, Simd::splat(1048576.0)));
    Reduced {
        quadrant: large.select(reduced.quadrant, medium.quadrant),
        hi: large.select(reduced.hi, medium.hi),
        lo: large.select(reduced.lo, medium.lo),
    }
}
