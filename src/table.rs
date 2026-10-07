use core::simd::{Simd, SimdElement};

#[inline(always)]
pub(crate) fn lookup_pairs<T: SimdElement + Default, const N: usize>(
    table: &[[T; 2]],
    index: Simd<usize, N>,
) -> (Simd<T, N>, Simd<T, N>) {
    let mut rows = [[T::default(); 2]; N];
    for lane in 0..N {
        Simd::from_array(table[index[lane]]).copy_to_slice(&mut rows[lane]);
    }
    let values = rows.as_flattened();
    let first = Simd::from_slice(&values[..N]);
    let second = Simd::from_slice(&values[N..]);
    first.deinterleave(second)
}
