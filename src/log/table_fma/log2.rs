/*
 * Derived from Arm optimized-routines math/aarch64/advsimd/log2.c; table: math/aarch64/v_log2_data.c, with changes.
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

const LOG2_POLY: [f64; 5] = [
    f64::from_bits(0xbfe71547652b8300), // -0x1.71547652b8300p-1
    f64::from_bits(0x3fdec709dc340953), // 0x1.ec709dc340953p-2
    f64::from_bits(0xbfd71547651c8f35), // -0x1.71547651c8f35p-2
    f64::from_bits(0x3fd2777ebe12dda5), // 0x1.2777ebe12dda5p-2
    f64::from_bits(0xbfcec738d616fe26), // -0x1.ec738d616fe26p-3
];

const LOG2_CENTER: [f64; 128] = [
    f64::from_bits(0xbfe00130d57f5fad),
    f64::from_bits(0xbfdf802661bd725e),
    f64::from_bits(0xbfdefea1c6f73a5b),
    f64::from_bits(0xbfde7dd1dcd06f05),
    f64::from_bits(0xbfddfdb4ae024809),
    f64::from_bits(0xbfdd7e484d101958),
    f64::from_bits(0xbfdcff8ad452f6e0),
    f64::from_bits(0xbfdc817a666c997f),
    f64::from_bits(0xbfdc04152d640419),
    f64::from_bits(0xbfdb87595a3f64b2),
    f64::from_bits(0xbfdb0b4526c44d07),
    f64::from_bits(0xbfda8fd6d1a90f5e),
    f64::from_bits(0xbfda150ca2559fc6),
    f64::from_bits(0xbfd99ae4e62cca29),
    f64::from_bits(0xbfd9215df1a1e842),
    f64::from_bits(0xbfd8a8761fe1f0d9),
    f64::from_bits(0xbfd8302bd1cc9a54),
    f64::from_bits(0xbfd7b87d6fb437f6),
    f64::from_bits(0xbfd741696673a86d),
    f64::from_bits(0xbfd6caee2b3c6fe4),
    f64::from_bits(0xbfd6550a3666c27a),
    f64::from_bits(0xbfd5dfbc08de02a4),
    f64::from_bits(0xbfd56b022766c84a),
    f64::from_bits(0xbfd4f6db1c955536),
    f64::from_bits(0xbfd4834579063054),
    f64::from_bits(0xbfd4103fd2249a76),
    f64::from_bits(0xbfd39dc8c3fe6dab),
    f64::from_bits(0xbfd32bdeed4b5c8f),
    f64::from_bits(0xbfd2ba80f41e20dd),
    f64::from_bits(0xbfd249ad8332f4a7),
    f64::from_bits(0xbfd1d96347e7f3eb),
    f64::from_bits(0xbfd169a0f7d6604a),
    f64::from_bits(0xbfd0fa654a221909),
    f64::from_bits(0xbfd08baefcf8251a),
    f64::from_bits(0xbfd01d7cd14deecd),
    f64::from_bits(0xbfcf5f9b1ad55495),
    f64::from_bits(0xbfce853ff76a77af),
    f64::from_bits(0xbfcdabe5d624cba1),
    f64::from_bits(0xbfccd38a5cef4822),
    f64::from_bits(0xbfcbfc2b38d315f9),
    f64::from_bits(0xbfcb25c61f5edd0f),
    f64::from_bits(0xbfca5058d18e9cac),
    f64::from_bits(0xbfc97be1113e47a3),
    f64::from_bits(0xbfc8a85cafdf5e27),
    f64::from_bits(0xbfc7d5c97e8fc45b),
    f64::from_bits(0xbfc704255d6486e4),
    f64::from_bits(0xbfc6336e2cedd7bf),
    f64::from_bits(0xbfc563a1d9b0cc6a),
    f64::from_bits(0xbfc494be541aaa6f),
    f64::from_bits(0xbfc3c6c1964dd0f2),
    f64::from_bits(0xbfc2f9a99f19a243),
    f64::from_bits(0xbfc22d7473444460),
    f64::from_bits(0xbfc1622020d4f7f5),
    f64::from_bits(0xbfc097aabb3553f3),
    f64::from_bits(0xbfbf9c24b48014c5),
    f64::from_bits(0xbfbe0aaa3bdc858a),
    f64::from_bits(0xbfbc7ae257c952d6),
    f64::from_bits(0xbfbaecc960a03e58),
    f64::from_bits(0xbfb9605bb724d541),
    f64::from_bits(0xbfb7d595ca7147ce),
    f64::from_bits(0xbfb64c74165002d9),
    f64::from_bits(0xbfb4c4f31c86d344),
    f64::from_bits(0xbfb33f0f70388258),
    f64::from_bits(0xbfb1bac5abb3037d),
    f64::from_bits(0xbfb0381272495f21),
    f64::from_bits(0xbfad6de4eba2de2a),
    f64::from_bits(0xbfaa6ec4e8156898),
    f64::from_bits(0xbfa772be542e3e1b),
    f64::from_bits(0xbfa479cadcde852d),
    f64::from_bits(0xbfa183e4265faa50),
    f64::from_bits(0xbf9d2207fdaa1b85),
    f64::from_bits(0xbf9742486cb4a6a2),
    f64::from_bits(0xbf91687d77cfc299),
    f64::from_bits(0xbf87293623a6b5de),
    f64::from_bits(0xbf770ec80ec8f25d),
    f64::from_bits(0x0),
    f64::from_bits(0x3f8704c1ca6b6bc9),
    f64::from_bits(0x3f96eac8ba664bea),
    f64::from_bits(0x3fa11e67d040772d),
    f64::from_bits(0x3fa6bc665e2105de),
    f64::from_bits(0x3fac4f8a9772bf1d),
    f64::from_bits(0x3fb0ebff10fbb951),
    f64::from_bits(0x3fb3aaf4d7805d11),
    f64::from_bits(0x3fb664ba81a4d717),
    f64::from_bits(0x3fb9196387da6de4),
    f64::from_bits(0x3fbbc902f2b77960),
    f64::from_bits(0x3fbe73ab5f584f28),
    f64::from_bits(0x3fc08cb78510d232),
    f64::from_bits(0x3fc1dd2fe2f0dcb5),
    f64::from_bits(0x3fc32b4784400df4),
    f64::from_bits(0x3fc47706f3d49942),
    f64::from_bits(0x3fc5c0768ee4a4dc),
    f64::from_bits(0x3fc7079e86fc7c6d),
    f64::from_bits(0x3fc84c86e1183467),
    f64::from_bits(0x3fc98f377a34b499),
    f64::from_bits(0x3fcacfb803bc924b),
    f64::from_bits(0x3fcc0e10098b025f),
    f64::from_bits(0x3fcd4a46efe103ef),
    f64::from_bits(0x3fce8463f45b8d0b),
    f64::from_bits(0x3fcfbc6e3228997f),
    f64::from_bits(0x3fd079364f2e5aa8),
    f64::from_bits(0x3fd1133306010a63),
    f64::from_bits(0x3fd1ac309631bd17),
    f64::from_bits(0x3fd24432485370c1),
    f64::from_bits(0x3fd2db3b5449132f),
    f64::from_bits(0x3fd3714ee1d7a320),
    f64::from_bits(0x3fd406700ab52c94),
    f64::from_bits(0x3fd49aa1d87522b2),
    f64::from_bits(0x3fd52de746d7ecb2),
    f64::from_bits(0x3fd5c0434336b343),
    f64::from_bits(0x3fd651b8ad6c90d1),
    f64::from_bits(0x3fd6e24a56ab5831),
    f64::from_bits(0x3fd771fb04ec29b1),
    f64::from_bits(0x3fd800cd6f19c25e),
    f64::from_bits(0x3fd88ec441df11df),
    f64::from_bits(0x3fd91be21b7c93f5),
    f64::from_bits(0x3fd9a8298f8c7454),
    f64::from_bits(0x3fda339d255c04dd),
    f64::from_bits(0x3fdabe3f59f43db7),
    f64::from_bits(0x3fdb48129deca9ef),
    f64::from_bits(0x3fdbd119575364c1),
    f64::from_bits(0x3fdc5955e23ebcbc),
    f64::from_bits(0x3fdce0ca8f4e1557),
    f64::from_bits(0x3fdd6779a5a75774),
    f64::from_bits(0x3fdded6563550d27),
    f64::from_bits(0x3fde728ffafd840e),
    f64::from_bits(0x3fdef6fb96c8d739),
    f64::from_bits(0x3fdf7aaa57907219),
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
    let center = { &LOG2_CENTER };
    let logc = Simd::gather_or(center, index, Simd::splat(0.0));
    let r = z.mul_add(invc, Simd::splat(-1.0));
    let hi = { r.mul_add(Simd::splat(core::f64::consts::LOG2_E), logc + k) };
    let c = { LOG2_POLY };
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

/// Computes the binary logarithm with FMA table reduction.
#[inline]
pub fn log2_f64<const N: usize>(x: Simd<f64, N>) -> Simd<f64, N> {
    log::<N>(x)
}
