// Local SIMD quadrant/sign finishing.

use core::simd::prelude::*;

use simd_macros::vectorize;

#[allow(unused_braces)]
pub(super) fn finish<const N: usize>(
    x: Simd<f64, N>,
    quadrant: Simd<u64, N>,
    s: Simd<f64, N>,
    c: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    vectorize!(N, {
        let sin = if quadrant & 1 == 0 { s } else { c };
        let sin = if quadrant & 2 == 0 { sin } else { -sin };
        let sin = <f64>::from_bits(sin.to_bits() ^ (x.to_bits() & (1u64 << 63)));
        let cos = if quadrant & 1 == 0 { c } else { s };
        let cos_quadrant = quadrant + 1;
        let cos = if cos_quadrant & 2 == 0 { cos } else { -cos };
        let finite = x.is_finite();
        let sin = if finite { sin } else { x - x };
        let cos = if finite { cos } else { x - x };
        (sin, cos)
    })
}
