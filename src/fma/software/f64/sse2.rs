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
    let split = Simd::splat(134217729.0); // 1 + 2^27
    let p = a * split;
    let ha = (a - p) + p;
    let la = a - ha;
    let p = b * split;
    let hb = (b - p) + p;
    let lb = b - hb;
    let p = ha * hb;
    let q = ha * lb + la * hb;
    let hi = p + q;
    let lo = ((p - hi) + q) + la * lb;
    (hi, lo)
}

#[inline]
fn add_adjusted<const N: usize>(a: Simd<f64, N>, b: Simd<f64, N>) -> Simd<f64, N> {
    let (hi, lo) = add(a, b);
    // Round the low-part addition to odd, preserving its discarded bits as
    // sticky information for the final addition to the high part.
    let bits = hi.to_bits();
    let adjust = lo.simd_ne(Simd::splat(0.0)) & (bits & Simd::splat(1)).simd_eq(Simd::splat(0));
    let step = Simd::splat(1) - (((bits ^ lo.to_bits()) >> 62) & Simd::splat(2));
    Simd::from_bits(adjust.select(bits + step, bits))
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
    if N > 1 && !cfg!(target_feature = "avx") {
        // SSE2 emulates the full-range vector integer operations, and its
        // vector range checks add overhead for mixed inputs. Reuse the
        // optimized scalar kernel per lane to retain scalar performance
        // across the full input range, including exceptional values.
        let x = x.to_array();
        let y = y.to_array();
        let z = z.to_array();
        return Simd::from_array(core::array::from_fn(|lane| {
            fma_f64::<1>(
                Simd::splat(x[lane]),
                Simd::splat(y[lane]),
                Simd::splat(z[lane]),
            )[0]
        }));
    }
    // These bounds keep every nonzero product, split and residual normal.
    // Even the smallest product bit is at least 2^-1006, above 2^-1022.
    // Inspect 32-bit exponent fields: SSE2 can compare these directly, and
    // integer range checks avoid FP assists for subnormal input lanes.
    let exponent = |v: Simd<f64, N>| (v.to_bits() >> 52).cast::<u32>() & Simd::splat(0x7ff);
    let bounded = (exponent(x) - Simd::splat(573)).simd_lt(Simd::splat(900))
        & (exponent(y) - Simd::splat(573)).simd_lt(Simd::splat(900))
        & (exponent(z) - Simd::splat(123)).simd_lt(Simd::splat(1800));
    if bounded.all() {
        double_double(x, y, z)
    } else {
        general(x, y, z, bounded)
    }
}

#[inline(always)]
fn general<const N: usize>(
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>,
    bounded: Mask<i32, N>,
) -> Simd<f64, N> {
    if N == 1 {
        super::fma_f64_integer(x, y, z)
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
                super::fma_f64_integer::<1>(
                    Simd::splat(x[lane]),
                    Simd::splat(y[lane]),
                    Simd::splat(z[lane]),
                )[0]
            }
        }))
    }
}
