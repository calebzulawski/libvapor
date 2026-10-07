/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log10.c; table: math/aarch64/v_log10_data.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2022-2025, Arm Limited.
 * Copyright (c) 2022-2024, Arm Limited.
 * SPDX-License-Identifier: MIT
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use core::simd::prelude::*;
use std::simd::StdFloat;

const LOG10_POLY: [f64; 5] = [
    f64::from_bits(0xbfcbcb7b1526e506), // -0x1.bcb7b1526e506p-3
    f64::from_bits(0x3fc287a7636be1d1), // 0x1.287a7636be1d1p-3
    f64::from_bits(0xbfbbcb7b158af938), // -0x1.bcb7b158af938p-4
    f64::from_bits(0x3fb63c78734e6d07), // 0x1.63c78734e6d07p-4
    f64::from_bits(0xbfb287461742fee4), // -0x1.287461742fee4p-4
];

const LOG10_CENTER: [f64; 128] = [
    f64::from_bits(0xbfc345825f221684),
    f64::from_bits(0xbfc2f71a1f0c554e),
    f64::from_bits(0xbfc2a91fdb30b1f4),
    f64::from_bits(0xbfc25b9260981a04),
    f64::from_bits(0xbfc20e7081762193),
    f64::from_bits(0xbfc1c1b914aeefac),
    f64::from_bits(0xbfc1756af5de404d),
    f64::from_bits(0xbfc12985059c90bf),
    f64::from_bits(0xbfc0de0628f63df4),
    f64::from_bits(0xbfc092ed492e08ee),
    f64::from_bits(0xbfc0483954caf1df),
    f64::from_bits(0xbfbffbd27a9adbc0),
    f64::from_bits(0xbfbf67f7f2e3d1a0),
    f64::from_bits(0xbfbed4e1071ceebe),
    f64::from_bits(0xbfbe428bb47413c4),
    f64::from_bits(0xbfbdb0f6003028d6),
    f64::from_bits(0xbfbd201df6749831),
    f64::from_bits(0xbfbc9001ac5c9672),
    f64::from_bits(0xbfbc009f3c78c790),
    f64::from_bits(0xbfbb71f4cb642e53),
    f64::from_bits(0xbfbae400818526b2),
    f64::from_bits(0xbfba56c091954f87),
    f64::from_bits(0xbfb9ca3332f096ee),
    f64::from_bits(0xbfb93e56a3f23e55),
    f64::from_bits(0xbfb8b3292a3903b0),
    f64::from_bits(0xbfb828a9112d9618),
    f64::from_bits(0xbfb79ed4ac35f5ac),
    f64::from_bits(0xbfb715aa51ed28c4),
    f64::from_bits(0xbfb68d2861c999e9),
    f64::from_bits(0xbfb6054d40ded210),
    f64::from_bits(0xbfb57e17576bc9a2),
    f64::from_bits(0xbfb4f7851798bb0b),
    f64::from_bits(0xbfb47194f5690ae3),
    f64::from_bits(0xbfb3ec456d58ec47),
    f64::from_bits(0xbfb36794ff3e5f55),
    f64::from_bits(0xbfb2e382315725e4),
    f64::from_bits(0xbfb2600b8ed82e91),
    f64::from_bits(0xbfb1dd2fa85efc12),
    f64::from_bits(0xbfb15aed136e3961),
    f64::from_bits(0xbfb0d94269d1a30d),
    f64::from_bits(0xbfb0582e4a7659f5),
    f64::from_bits(0xbfafaf5eb655742d),
    f64::from_bits(0xbfaeaf888487e8ee),
    f64::from_bits(0xbfadb0d75ef25a82),
    f64::from_bits(0xbfacb348a49e6431),
    f64::from_bits(0xbfabb6d9c69acdd8),
    f64::from_bits(0xbfaabb88368aa7a0),
    f64::from_bits(0xbfa9c1517476af14),
    f64::from_bits(0xbfa8c833051bfa4d),
    f64::from_bits(0xbfa7d02a78e7fb31),
    f64::from_bits(0xbfa6d93565e97c5f),
    f64::from_bits(0xbfa5e351695db0c5),
    f64::from_bits(0xbfa4ee7c2ba67adc),
    f64::from_bits(0xbfa3fab35ba16c01),
    f64::from_bits(0xbfa307f4ad854bc9),
    f64::from_bits(0xbfa2163ddf4f988c),
    f64::from_bits(0xbfa1258cb5d19e22),
    f64::from_bits(0xbfa035defdba3188),
    f64::from_bits(0xbf9e8e651191bce4),
    f64::from_bits(0xbf9cb30a62be444c),
    f64::from_bits(0xbf9ad9a9b3043823),
    f64::from_bits(0xbf99023ecda1ccde),
    f64::from_bits(0xbf972cc592bd82d0),
    f64::from_bits(0xbf955939eb1f9c6e),
    f64::from_bits(0xbf938797ca6cc5a0),
    f64::from_bits(0xbf91b7db35c2c072),
    f64::from_bits(0xbf8fd400812ee9a2),
    f64::from_bits(0xbf8c3c05fb4620f1),
    f64::from_bits(0xbf88a7bf3c40e2e3),
    f64::from_bits(0xbf8517249c15a75c),
    f64::from_bits(0xbf818a2ea5330c91),
    f64::from_bits(0xbf7c01abc8cdc4e2),
    f64::from_bits(0xbf74f6261750dec9),
    f64::from_bits(0xbf6be37b6612afa7),
    f64::from_bits(0xbf5bc3a8398ac260),
    f64::from_bits(0x0),
    f64::from_bits(0x3f6bb796219f30a5),
    f64::from_bits(0x3f7b984fdcba61ce),
    f64::from_bits(0x3f849cf12adf8e8c),
    f64::from_bits(0x3f8b6075b5217083),
    f64::from_bits(0x3f910b7466fc30dd),
    f64::from_bits(0x3f94603e4db6a3a1),
    f64::from_bits(0x3f97aeb10e99e105),
    f64::from_bits(0x3f9af6e49b0f0e36),
    f64::from_bits(0x3f9e38f064f41179),
    f64::from_bits(0x3fa0ba75abbb7623),
    f64::from_bits(0x3fa25575ee2dba86),
    f64::from_bits(0x3fa3ed83f477f946),
    f64::from_bits(0x3fa582aa79af60ef),
    f64::from_bits(0x3fa714f400fa83ae),
    f64::from_bits(0x3fa8a46ad3901cb9),
    f64::from_bits(0x3faa311903b6b870),
    f64::from_bits(0x3fabbb086f216911),
    f64::from_bits(0x3fad4242bdda648e),
    f64::from_bits(0x3faec6d167c2af10),
    f64::from_bits(0x3fb0245ed8221426),
    f64::from_bits(0x3fb0e40856c74f64),
    f64::from_bits(0x3fb1a269a31120fe),
    f64::from_bits(0x3fb25f8718fc076c),
    f64::from_bits(0x3fb31b64ffc95bf0),
    f64::from_bits(0x3fb3d60787ca5063),
    f64::from_bits(0x3fb48f72ccd187fd),
    f64::from_bits(0x3fb547aad6602f1c),
    f64::from_bits(0x3fb5feb3989d3acb),
    f64::from_bits(0x3fb6b490f3978c79),
    f64::from_bits(0x3fb76946b3f5e703),
    f64::from_bits(0x3fb81cd895717c83),
    f64::from_bits(0x3fb8cf4a4055c30e),
    f64::from_bits(0x3fb9809f4c48c0eb),
    f64::from_bits(0x3fba30db3f9899ef),
    f64::from_bits(0x3fbae001905458fc),
    f64::from_bits(0x3fbb8e15a2e3a2cd),
    f64::from_bits(0x3fbc3b1ace2b0996),
    f64::from_bits(0x3fbce71456edfa62),
    f64::from_bits(0x3fbd9205759882c4),
    f64::from_bits(0x3fbe3bf1513af0df),
    f64::from_bits(0x3fbee4db0412c414),
    f64::from_bits(0x3fbf8cc5998de3a5),
    f64::from_bits(0x3fc019da085eaeb1),
    f64::from_bits(0x3fc06cd4acdb4e3d),
    f64::from_bits(0x3fc0bf542bef813f),
    f64::from_bits(0x3fc11159f14da262),
    f64::from_bits(0x3fc162e761c10d1c),
    f64::from_bits(0x3fc1b3fddc60d43e),
    f64::from_bits(0x3fc2049ebac86aa6),
    f64::from_bits(0x3fc254cb4fb7836a),
    f64::from_bits(0x3fc2a484e8d0d252),
    f64::from_bits(0x3fc2f3ccce1c860b),
];

const INVC: [f64; 128] = [
    f64::from_bits(0x3ff6a133d0dec120),
    f64::from_bits(0x3ff6815f2f3e42ed),
    f64::from_bits(0x3ff661e39be1ac9e),
    f64::from_bits(0x3ff642bfa30ac371),
    f64::from_bits(0x3ff623f1d916f323),
    f64::from_bits(0x3ff60578da220f65),
    f64::from_bits(0x3ff5e75349dea571),
    f64::from_bits(0x3ff5c97fd387a75a),
    f64::from_bits(0x3ff5abfd2981f200),
    f64::from_bits(0x3ff58eca051dc99c),
    f64::from_bits(0x3ff571e526d9df12),
    f64::from_bits(0x3ff5554d555b3fcb),
    f64::from_bits(0x3ff539015e2a20cd),
    f64::from_bits(0x3ff51d0014ee0164),
    f64::from_bits(0x3ff50148538cd9ee),
    f64::from_bits(0x3ff4e5d8f9f698a1),
    f64::from_bits(0x3ff4cab0edca66be),
    f64::from_bits(0x3ff4afcf1a9db874),
    f64::from_bits(0x3ff495327136e16f),
    f64::from_bits(0x3ff47ad9e84af28f),
    f64::from_bits(0x3ff460c47b39ae15),
    f64::from_bits(0x3ff446f12b278001),
    f64::from_bits(0x3ff42d5efdd720ec),
    f64::from_bits(0x3ff4140cfe001a0f),
    f64::from_bits(0x3ff3fafa3b421f69),
    f64::from_bits(0x3ff3e225c9c8ece5),
    f64::from_bits(0x3ff3c98ec29a211a),
    f64::from_bits(0x3ff3b13442a413fe),
    f64::from_bits(0x3ff399156baa3c54),
    f64::from_bits(0x3ff38131639b4cdb),
    f64::from_bits(0x3ff36987540fbf53),
    f64::from_bits(0x3ff352166b648f61),
    f64::from_bits(0x3ff33adddb3eb575),
    f64::from_bits(0x3ff323dcd99fc1d3),
    f64::from_bits(0x3ff30d129fefc7d2),
    f64::from_bits(0x3ff2f67e6b72fe7d),
    f64::from_bits(0x3ff2e01f7cf8b187),
    f64::from_bits(0x3ff2c9f518ddc86e),
    f64::from_bits(0x3ff2b3fe86e5f413),
    f64::from_bits(0x3ff29e3b1211b25c),
    f64::from_bits(0x3ff288aa08b373cf),
    f64::from_bits(0x3ff2734abcaa8467),
    f64::from_bits(0x3ff25e1c82459b81),
    f64::from_bits(0x3ff2491eb1ad59c5),
    f64::from_bits(0x3ff23450a54048b5),
    f64::from_bits(0x3ff21fb1bb09e578),
    f64::from_bits(0x3ff20b415346d8f7),
    f64::from_bits(0x3ff1f6fed179a1ac),
    f64::from_bits(0x3ff1e2e99b93c7b3),
    f64::from_bits(0x3ff1cf011a7a882a),
    f64::from_bits(0x3ff1bb44b97dba5a),
    f64::from_bits(0x3ff1a7b3e66cdd4f),
    f64::from_bits(0x3ff1944e11dc56cd),
    f64::from_bits(0x3ff18112aebb1a6e),
    f64::from_bits(0x3ff16e013231b7e9),
    f64::from_bits(0x3ff15b1913f156cf),
    f64::from_bits(0x3ff14859cdedde13),
    f64::from_bits(0x3ff135c2dc68cfa4),
    f64::from_bits(0x3ff12353bdb01684),
    f64::from_bits(0x3ff1110bf25b85b4),
    f64::from_bits(0x3ff0feeafd2f8577),
    f64::from_bits(0x3ff0ecf062c51c3b),
    f64::from_bits(0x3ff0db1baa076c8b),
    f64::from_bits(0x3ff0c96c5bb3048e),
    f64::from_bits(0x3ff0b7e20263e070),
    f64::from_bits(0x3ff0a67c2acd0ce3),
    f64::from_bits(0x3ff0953a6391e982),
    f64::from_bits(0x3ff0841c3caea380),
    f64::from_bits(0x3ff07321489b13ea),
    f64::from_bits(0x3ff062491aee9904),
    f64::from_bits(0x3ff05193497a7cc5),
    f64::from_bits(0x3ff040ff6b5f5e9f),
    f64::from_bits(0x3ff0308d19aa6127),
    f64::from_bits(0x3ff0203beedb0c67),
    f64::from_bits(0x3ff010037d38bcc2),
    f64::from_bits(0x3ff0000000000000),
    f64::from_bits(0x3fefc06d493cca10),
    f64::from_bits(0x3fef81e6ac3b918f),
    f64::from_bits(0x3fef44546ef18996),
    f64::from_bits(0x3fef07b10382c84b),
    f64::from_bits(0x3feecbf7070e59d4),
    f64::from_bits(0x3fee91213f715939),
    f64::from_bits(0x3fee572a9a75f7b7),
    f64::from_bits(0x3fee1e0e2c530207),
    f64::from_bits(0x3fede5c72d8a8be3),
    f64::from_bits(0x3fedae50fa5658cc),
    f64::from_bits(0x3fed77a71145a2da),
    f64::from_bits(0x3fed41c51166623e),
    f64::from_bits(0x3fed0ca6ba0bb29f),
    f64::from_bits(0x3fecd847e8e59681),
    f64::from_bits(0x3feca4a499693e00),
    f64::from_bits(0x3fec71b8e399e821),
    f64::from_bits(0x3fec3f80faf19077),
    f64::from_bits(0x3fec0df92dc2b0ec),
    f64::from_bits(0x3febdd1de3cbb542),
    f64::from_bits(0x3febaceb9e1007a3),
    f64::from_bits(0x3feb7d5ef543e55e),
    f64::from_bits(0x3feb4e749977d953),
    f64::from_bits(0x3feb20295155478e),
    f64::from_bits(0x3feaf279f8e82be2),
    f64::from_bits(0x3feac5638197fdf3),
    f64::from_bits(0x3fea98e2f102e087),
    f64::from_bits(0x3fea6cf5606d05c1),
    f64::from_bits(0x3fea4197fc04d746),
    f64::from_bits(0x3fea16c80293dc01),
    f64::from_bits(0x3fe9ec82c4dc5bc9),
    f64::from_bits(0x3fe9c2c5a491f534),
    f64::from_bits(0x3fe9998e1480b618),
    f64::from_bits(0x3fe970d9977c6c2d),
    f64::from_bits(0x3fe948a5c023d212),
    f64::from_bits(0x3fe920f0303d6809),
    f64::from_bits(0x3fe8f9b698a98b45),
    f64::from_bits(0x3fe8d2f6b81726f6),
    f64::from_bits(0x3fe8acae5bb55bad),
    f64::from_bits(0x3fe886db5d9275b8),
    f64::from_bits(0x3fe8617ba567c13c),
    f64::from_bits(0x3fe83c8d27487800),
    f64::from_bits(0x3fe8180de3c5dbe7),
    f64::from_bits(0x3fe7f3fbe71cdb71),
    f64::from_bits(0x3fe7d055498071c1),
    f64::from_bits(0x3fe7ad182e54f65a),
    f64::from_bits(0x3fe78a42c3c90125),
    f64::from_bits(0x3fe767d342f76944),
    f64::from_bits(0x3fe745c7ef26b00a),
    f64::from_bits(0x3fe7241f15769d0f),
    f64::from_bits(0x3fe702d70d396e41),
    f64::from_bits(0x3fe6e1ee3700cd11),
    f64::from_bits(0x3fe6c162fc9cbe02),
];

#[inline]
fn kernel<const N: usize>(bits: Simd<u64, N>) -> Simd<f64, N> {
    let u = bits - Simd::splat(0x3fe6900900000000);
    let k = (u.cast::<i64>() >> 52).cast::<i32>().cast::<f64>();
    let z = Simd::<f64, N>::from_bits(bits - (u & Simd::splat(0xfff0000000000000)));
    let index = ((u >> 45) & Simd::splat(127)).cast::<usize>();
    let table = &const { super::interleave(&INVC, &LOG10_CENTER) };
    let (invc, logc) = crate::table::lookup_pairs(table, index);
    let r = z.mul_add(invc, Simd::splat(-1.0));
    let hi = {
        let hi = r.mul_add(Simd::splat(core::f64::consts::LOG10_E), logc);
        k.mul_add(Simd::splat(core::f64::consts::LOG10_2), hi)
    };
    let c = { LOG10_POLY };
    let c = c.map(Simd::splat);
    let r2 = r * r;
    let y = r.mul_add(c[3], c[2]);
    let p = r.mul_add(c[1], c[0]);
    let y = r2.mul_add(c[4], y);
    let y = r2.mul_add(y, p);
    r2.mul_add(y, hi)
}

#[cold]
#[inline(never)]
fn special<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let bits = super::super::reduction::normalize_f64(x);
    let y = kernel::<N>(bits);
    super::super::reduction::finish_f64(x, y)
}

#[inline]
fn log<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    let regular = x.simd_ge(Simd::splat(f64::MIN_POSITIVE)) & x.simd_lt(Simd::splat(f64::INFINITY));
    if regular.all() {
        kernel::<N>(x.to_bits())
    } else {
        special::<N>(x)
    }
}

/// Computes the decimal logarithm with FMA table reduction.
#[inline]
pub fn log10_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    log::<N>(x)
}
