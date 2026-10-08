// Local SIMD quadrant/sign finishing.

use core::simd::prelude::*;

pub(super) fn finish<const N: usize>(
    x: Simd<f64, N>,
    quadrant: Simd<u64, N>,
    s: Simd<f64, N>,
    c: Simd<f64, N>,
) -> (Simd<f64, N>, Simd<f64, N>) {
    let sin = (quadrant & Simd::splat(1))
        .simd_eq(Simd::splat(0))
        .select(s, c);
    let sin = (quadrant & Simd::splat(2))
        .simd_eq(Simd::splat(0))
        .select(sin, -sin);
    let sin = Simd::<f64, N>::from_bits(sin.to_bits() ^ (x.to_bits() & (Simd::splat(1u64) << 63)));
    let cos = (quadrant & Simd::splat(1))
        .simd_eq(Simd::splat(0))
        .select(c, s);
    let cos_quadrant = quadrant + Simd::splat(1);
    let cos = (cos_quadrant & Simd::splat(2))
        .simd_eq(Simd::splat(0))
        .select(cos, -cos);
    let finite = x.is_finite();
    let sin = finite.select(sin, x - x);
    let cos = finite.select(cos, x - x);
    (sin, cos)
}
