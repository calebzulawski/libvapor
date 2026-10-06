use core::simd::Simd;

#[path = "f64/double_double.rs"]
mod double_double;
#[path = "f64/integer.rs"]
mod integer;

/// Computes the correctly rounded fused multiply-add without an FMA instruction.
/// Assumes round-to-nearest, ties-to-even.
#[inline]
pub fn fma_f64<const N: usize>(x: Simd<f64, N>, y: Simd<f64, N>, z: Simd<f64, N>) -> Simd<f64, N> {
    // Double-double arithmetic avoids costly vector integer operations on
    // SSE2 and Wasm, while retaining the integer kernel for full-range lanes.
    // Wasm relaxed multiply-add permits unfused results, so it cannot replace
    // this correctly rounded software implementation.
    if cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2",
            not(target_feature = "avx512f")
        ),
        target_arch = "wasm32",
        target_arch = "wasm64"
    )) {
        double_double::fma_f64(x, y, z)
    } else {
        integer::fma_f64(x, y, z)
    }
}

#[cfg(test)]
#[path = "f64/tests.rs"]
mod tests;
