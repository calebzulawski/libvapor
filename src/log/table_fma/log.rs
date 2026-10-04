/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log.c; table: math/aarch64/v_log_data.c, with changes.
 */

/*
 * Copyright (c) 1999-2022, Arm Limited.
 * Copyright (c) 2019-2025, Arm Limited.
 * Copyright (c) 2019-2024, Arm Limited.
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

const LOG_POLY: [f64; 5] = [
    f64::from_bits(0xbfdffffffffffff7), // -0x1.ffffffffffff7p-2
    f64::from_bits(0x3fd55555555170d4), // 0x1.55555555170d4p-2
    f64::from_bits(0xbfd0000000399c27), // -0x1.0000000399c27p-2
    f64::from_bits(0x3fc999b2e90e94ca), // 0x1.999b2e90e94cap-3
    f64::from_bits(0xbfc554e550bd501e), // -0x1.554e550bd501ep-3
];

const LOG_CENTER: [f64; 128] = [
    f64::from_bits(0xbfd62fe995eb963a),
    f64::from_bits(0xbfd5d5a48dad6b67),
    f64::from_bits(0xbfd57bde257d2769),
    f64::from_bits(0xbfd52294fbf2af55),
    f64::from_bits(0xbfd4c9c7b598aa38),
    f64::from_bits(0xbfd47174fc5ff560),
    f64::from_bits(0xbfd4199b7fa7b5ca),
    f64::from_bits(0xbfd3c239f48cfb99),
    f64::from_bits(0xbfd36b4f154d2aeb),
    f64::from_bits(0xbfd314d9a0ff32fb),
    f64::from_bits(0xbfd2bed85cca3cff),
    f64::from_bits(0xbfd2694a11421af9),
    f64::from_bits(0xbfd2142d8d014fb2),
    f64::from_bits(0xbfd1bf81a2c77776),
    f64::from_bits(0xbfd16b452a39c6a4),
    f64::from_bits(0xbfd11776ffa6c67e),
    f64::from_bits(0xbfd0c416035020e0),
    f64::from_bits(0xbfd071211aa10fda),
    f64::from_bits(0xbfd01e972e293b1b),
    f64::from_bits(0xbfcf98ee587fd434),
    f64::from_bits(0xbfcef5800ad716fb),
    f64::from_bits(0xbfce52e160484698),
    f64::from_bits(0xbfcdb1104b19352e),
    f64::from_bits(0xbfcd100ac59e0bd6),
    f64::from_bits(0xbfcc6fced287c3bd),
    f64::from_bits(0xbfcbd05a7b317c29),
    f64::from_bits(0xbfcb31abd229164f),
    f64::from_bits(0xbfca93c0edadb0a3),
    f64::from_bits(0xbfc9f697ee30d7dd),
    f64::from_bits(0xbfc95a2efa9aa40a),
    f64::from_bits(0xbfc8be843d796044),
    f64::from_bits(0xbfc82395ecc477ed),
    f64::from_bits(0xbfc7896240966422),
    f64::from_bits(0xbfc6efe77aca8c55),
    f64::from_bits(0xbfc65723e117ec5c),
    f64::from_bits(0xbfc5bf15c0955706),
    f64::from_bits(0xbfc527bb6c111da1),
    f64::from_bits(0xbfc491133c939f8f),
    f64::from_bits(0xbfc3fb1b90c7fc58),
    f64::from_bits(0xbfc365d2cc485f8d),
    f64::from_bits(0xbfc2d13758970de7),
    f64::from_bits(0xbfc23d47a721fd47),
    f64::from_bits(0xbfc1aa0229f25ec2),
    f64::from_bits(0xbfc117655ddebc3b),
    f64::from_bits(0xbfc0856fbf83ab6b),
    f64::from_bits(0xbfbfe83fabbaa106),
    f64::from_bits(0xbfbec6e8507a56cd),
    f64::from_bits(0xbfbda6d68c7cc2ea),
    f64::from_bits(0xbfbc88078462be0c),
    f64::from_bits(0xbfbb6a786a423565),
    f64::from_bits(0xbfba4e2676ac7f85),
    f64::from_bits(0xbfb9330eea777e76),
    f64::from_bits(0xbfb8192f134d5ad9),
    f64::from_bits(0xbfb70084464f0538),
    f64::from_bits(0xbfb5e90bdec5cb1f),
    f64::from_bits(0xbfb4d2c3433c5536),
    f64::from_bits(0xbfb3bda7e219879a),
    f64::from_bits(0xbfb2a9b732d27194),
    f64::from_bits(0xbfb196eeb2b10807),
    f64::from_bits(0xbfb0854be8ef8a7e),
    f64::from_bits(0xbfaee998cb277432),
    f64::from_bits(0xbfaccadb79919fb9),
    f64::from_bits(0xbfaaae5b1d8618b0),
    f64::from_bits(0xbfa89413015d7442),
    f64::from_bits(0xbfa67bfe7bf158de),
    f64::from_bits(0xbfa46618f83941be),
    f64::from_bits(0xbfa2525df1b0618a),
    f64::from_bits(0xbfa040c8e2f77c6a),
    f64::from_bits(0xbf9c62aad39f738a),
    f64::from_bits(0xbf9847fe3bdead9c),
    f64::from_bits(0xbf943183683400ac),
    f64::from_bits(0xbf901f31c4e1d544),
    f64::from_bits(0xbf882201d1e6b69a),
    f64::from_bits(0xbf800dd0f3e1bfd6),
    f64::from_bits(0xbf6ff6fe1feb4e53),
    f64::from_bits(0x0),
    f64::from_bits(0x3f7fe91885ec8e20),
    f64::from_bits(0x3f8fc516f716296d),
    f64::from_bits(0x3f97bb4dd70a015b),
    f64::from_bits(0x3f9f84c99b34b674),
    f64::from_bits(0x3fa39f9ce4fb2d71),
    f64::from_bits(0x3fa7756c0fd22e78),
    f64::from_bits(0x3fab43ee82db8f3a),
    f64::from_bits(0x3faf0b3fced60034),
    f64::from_bits(0x3fb165bd78d4878e),
    f64::from_bits(0x3fb3425d2715ebe6),
    f64::from_bits(0x3fb51b8bd91b7915),
    f64::from_bits(0x3fb6f15632c76a47),
    f64::from_bits(0x3fb8c3c88ecbe503),
    f64::from_bits(0x3fba92ef077625da),
    f64::from_bits(0x3fbc5ed5745fa006),
    f64::from_bits(0x3fbe27876de1c993),
    f64::from_bits(0x3fbfed104fce4cdc),
    f64::from_bits(0x3fc0d7bd9c17d78b),
    f64::from_bits(0x3fc1b76986cef97b),
    f64::from_bits(0x3fc295913d24f750),
    f64::from_bits(0x3fc37239fa295d17),
    f64::from_bits(0x3fc44d68dd78714b),
    f64::from_bits(0x3fc52722ebe5d780),
    f64::from_bits(0x3fc5ff6d12671f98),
    f64::from_bits(0x3fc6d64c2389484b),
    f64::from_bits(0x3fc7abc4da40fdda),
    f64::from_bits(0x3fc87fdbda1e8452),
    f64::from_bits(0x3fc95295b06a5f37),
    f64::from_bits(0x3fca23f6d34abbc5),
    f64::from_bits(0x3fcaf403a28e04f2),
    f64::from_bits(0x3fcbc2c06a85721a),
    f64::from_bits(0x3fcc903161240163),
    f64::from_bits(0x3fcd5c5aa93287eb),
    f64::from_bits(0x3fce274051823fa9),
    f64::from_bits(0x3fcef0e656300c16),
    f64::from_bits(0x3fcfb9509f05aa2a),
    f64::from_bits(0x3fd04041821f37af),
    f64::from_bits(0x3fd0a340a49b3029),
    f64::from_bits(0x3fd105a7918a126d),
    f64::from_bits(0x3fd1677819812b84),
    f64::from_bits(0x3fd1c8b405b40c0e),
    f64::from_bits(0x3fd2295d16cfa6b1),
    f64::from_bits(0x3fd28975066318a2),
    f64::from_bits(0x3fd2e8fd855d86fc),
    f64::from_bits(0x3fd347f83d605e59),
    f64::from_bits(0x3fd3a666d1244588),
    f64::from_bits(0x3fd4044adb6f8ec4),
    f64::from_bits(0x3fd461a5f077558c),
    f64::from_bits(0x3fd4be799e20b9c8),
    f64::from_bits(0x3fd51ac76a6b79df),
    f64::from_bits(0x3fd57690d5744a45),
    f64::from_bits(0x3fd5d1d758e45217),
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
    let invc = Simd::gather_or(&INVC, index, Simd::splat(0.0));
    let center = { &LOG_CENTER };
    let logc = Simd::gather_or(center, index, Simd::splat(0.0));
    let r = z.mul_add(invc, Simd::splat(-1.0));
    let hi = { k.mul_add(Simd::splat(core::f64::consts::LN_2), logc + r) };
    let c = { LOG_POLY };
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

/// Computes the natural logarithm with FMA table reduction.
#[inline]
pub fn log_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    log::<N>(x)
}
