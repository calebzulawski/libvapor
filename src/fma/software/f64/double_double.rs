/*
 * Derived from musl v1.1.16 src/math/fma.c, with changes.
 * Copyright and license notices inherited from musl follow.
 * SPDX-License-Identifier: MIT AND BSD-2-Clause
 */

/*
 * Copyright (c) 2005-2011 David Schultz <das@FreeBSD.ORG>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

/*
 * Copyright © 2005-2020 Rich Felker, et al.
 * Copyright © 2005-2014 Rich Felker, et al.
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use core::simd::prelude::*;

#[inline]
fn add<const N: usize>(a: Simd<f64, N>, b: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    let hi = a + b;
    let s = hi - a;
    let lo = (a - (hi - s)) + (b - s);
    (hi, lo)
}

#[inline]
fn multiply<const N: usize>(a: Simd<f64, N>, b: Simd<f64, N>) -> (Simd<f64, N>, Simd<f64, N>) {
    // Product split: Dekker, A Floating-Point Technique for Extending the
    // Available Precision, Numer. Math. 18, 224-242 (1971), as cited upstream.
    // Round the significands to 26 bits with integer operations. Either
    // direction at a midpoint gives an exact split, so ties may round up.
    let split = |v: Simd<f64, N>| {
        Simd::from_bits((v.to_bits() + Simd::splat(1 << 26)) & Simd::splat(!((1 << 27) - 1)))
    };
    let ha = split(a);
    let la = a - ha;
    let hb = split(b);
    let lb = b - hb;
    // Veltkamp's exact-product variant, described in Dekker's report,
    // section 5, p. 20: https://ir.cwi.nl/pub/9158/9158D.pdf
    // Compute the rounded product directly so the addend's sum can start
    // while the exact product residual is still being evaluated.
    let hi = a * b;
    let lo = (((ha * hb - hi) + ha * lb) + la * hb) + la * lb;
    (hi, lo)
}

#[inline]
fn add_adjusted<const N: usize>(a: Simd<f64, N>, b: Simd<f64, N>) -> Simd<f64, N> {
    let (hi, lo) = add(a, b);
    // Round the low-part addition to odd, preserving its discarded bits as
    // sticky information for the final addition to the high part.
    let bits = hi.to_bits();
    // An all-ones mask for even significands avoids an emulated SSE2
    // 64-bit comparison; odd significands need no adjustment.
    let even = (bits & Simd::splat(1)) - Simd::splat(1);
    let step = Simd::splat(1) - (((bits ^ lo.to_bits()) >> 62) & Simd::splat(2));
    let step = lo
        .simd_ne(Simd::splat(0.0))
        .select(even & step, Simd::splat(0));
    Simd::from_bits(bits + step)
}

#[inline]
fn double_double<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
) -> Simd<f64, N> {
    let (hi, lo) = multiply(x, y);
    let (sum, err) = add(hi, z);
    sum + add_adjusted(err, lo)
}

#[inline]
pub(super) fn fma_f64<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
) -> Simd<f64, N> {
    // These bounds keep every nonzero product, split and residual normal.
    // Even the smallest product bit is at least 2^-1006, above 2^-1022.
    // Inspect 32-bit exponent fields: SSE2 can compare these directly, and
    // integer range checks avoid FP assists for subnormal input lanes.
    let in_range = |v: Simd<f64, N>, first: i32, count: i32| {
        // Leave the exponent in the high 32-bit word to avoid shifting and
        // narrowing 64-bit lanes. Bias the unsigned range into one signed
        // comparison, which SSE2 supports without another sign-bit XOR.
        let exponent = (v.to_bits() >> 32).cast::<i32>() & Simd::splat(0x7ff00000);
        (exponent + Simd::splat(i32::MIN.wrapping_sub(first << 20)))
            .simd_lt(Simd::splat(i32::MIN + (count << 20)))
    };
    let bounded =
        in_range(x, 1023 - 450, 900) & in_range(y, 1023 - 450, 900) & in_range(z, 1023 - 900, 1800);
    if bounded.all() {
        double_double(x, y, z)
    } else {
        general(x, y, z, bounded)
    }
}

#[cold]
#[inline(never)]
fn general<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
    bounded: Mask<i32, N>,
) -> Simd<f64, N> {
    if N == 1 {
        super::integer::fma_f64(x, y, z)
    } else {
        let bounded = bounded.to_array();
        // Use the scalar integer kernel for full-range lanes, and retain
        // double-double arithmetic for the bounded lanes.
        let x = x.to_array();
        let y = y.to_array();
        let z = z.to_array();
        Simd::from_array(core::array::from_fn(|lane| {
            if bounded[lane] {
                double_double::<1>(
                    Simd::splat(x[lane]),
                    Simd::splat(y[lane]),
                    Simd::splat(z[lane]),
                )[0]
            } else {
                super::integer::fma_f64::<1>(
                    Simd::splat(x[lane]),
                    Simd::splat(y[lane]),
                    Simd::splat(z[lane]),
                )[0]
            }
        }))
    }
}
