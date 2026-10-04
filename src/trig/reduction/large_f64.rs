// Local Payne-Hanek reduction; 2/pi table: musl src/math/__rem_pio2_large.c.

use super::Reduced;

use core::simd::prelude::*;

use simd_macros::vectorize;

use super::super::data::*;

#[allow(unused_braces)]
pub(super) fn reduce_large_f64<const N: usize>(ax: Simd<f64, N>) -> Reduced<N> {
    let (index, shift, m0, m1) = vectorize!(N, {
        let bits = ax.to_bits();
        let exponent = (bits >> 52) - 1023;
        let mantissa = (bits & 0x000fffffffffffff) | 0x0010000000000000;
        // x = mantissa * 2^(exponent-52). Retain 256 fractional bits of
        // x*2/pi; the table truncation contributes less than 2^-203.
        let end = exponent + 204;
        let index = (end >> 5) + 2;
        let shift = end & 31;
        (index, shift, mantissa & 0xffffffff, mantissa >> 32)
    });

    // Multiply an exponent-selected 288-bit table window by the two-word
    // significand. Each product fits u64; carrying in base 2^32 is exact.
    let mut words = [Simd::splat(0u64); 9];
    let mut carry = Simd::splat(0u64);
    let mut previous = Simd::splat(0u64);
    for (i, word) in words.iter_mut().enumerate() {
        let (product, next_carry, current) = vectorize!(N, {
            let j = index - scalar!(i as u64);
            let j = j as usize;
            let a = <u32>::gather_or(&TWO_OVER_PI_F64, j, 0) as u64;
            let b = <u32>::gather_or(&TWO_OVER_PI_F64, j + 1, 0) as u64;
            let current = ((a << shift) | (b >> (32 - shift))) & 0xffffffff;
            let p0 = current * m0;
            let p1 = previous * m1;
            let sum = (p0 & 0xffffffff) + (p1 & 0xffffffff) + carry;
            let next_carry = (p0 >> 32) + (p1 >> 32) + (sum >> 32);
            (sum & 0xffffffff, next_carry, current)
        });
        *word = product;
        carry = next_carry;
        previous = current;
    }
    let round_up = words[7].simd_ge(Simd::splat(0x80000000));
    let quadrant = (words[8] + round_up.select(Simd::splat(1), Simd::splat(0))) & Simd::splat(3);

    // When rounding upward, complement the fractional integer before any
    // float conversion. This avoids cancellation in 1 - fraction.
    let mut carry = Simd::splat(1u64);
    for word in &mut words[..8] {
        let current = *word;
        let (magnitude, next_carry) = vectorize!(N, {
            let complement = (current ^ 0xffffffff) + carry;
            let magnitude = if round_up {
                complement & 0xffffffff
            } else {
                current
            };
            (magnitude, complement >> 32)
        });
        *word = magnitude;
        carry = next_carry;
    }

    // Sum the fixed-point magnitude into a double-double fraction, retaining
    // the rounding residual of each addition, including near-zero remainders.
    let mut hi = Simd::splat(0.0);
    let mut lo = Simd::splat(0.0);
    for i in (0..8).rev() {
        let word = words[i];
        let scale = f64::from_bits((1023 - 32 * (8 - i) as u64) << 52);
        (hi, lo) = vectorize!(N, {
            let term = word as f64 * scalar!(scale);
            let sum = hi + term;
            (sum, lo + (term - (sum - hi)))
        });
    }
    let (hi, lo) = vectorize!(N, {
        let fraction = hi + lo;
        let fraction_lo = (hi - fraction) + lo;

        // Dekker's product residual preserves the correction term without
        // requiring hardware FMA or a scalar math-library call.
        let split = 134217729.0 * fraction;
        let a_hi = split - (split - fraction);
        let a_lo = fraction - a_hi;
        let product = fraction * scalar!(PIO2);
        let error = ((a_hi * scalar!(PIO2_SPLIT_HI) - product)
            + a_hi * scalar!(PIO2_SPLIT_LO)
            + a_lo * scalar!(PIO2_SPLIT_HI))
            + a_lo * scalar!(PIO2_SPLIT_LO);
        let tail = fraction_lo * scalar!(PIO2) + fraction * scalar!(PIO2_TAIL) + error;
        let hi = product + tail;
        let lo = (product - hi) + tail;
        let hi = if round_up { -hi } else { hi };
        let lo = if round_up { -lo } else { lo };
        (hi, lo)
    });
    Reduced { quadrant, hi, lo }
}
