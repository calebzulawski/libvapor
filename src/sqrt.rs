use core::simd::{prelude::*, LaneCount, SupportedLaneCount};
use simd_macros::vectorize;

const RSQRT_TAB: [u32; 128] = [
    0xb451, 0xb2f0, 0xb196, 0xb044, 0xaef9, 0xadb6, 0xac79, 0xab43, 0xaa14, 0xa8eb, 0xa7c8, 0xa6aa,
    0xa592, 0xa480, 0xa373, 0xa26b, 0xa168, 0xa06a, 0x9f70, 0x9e7b, 0x9d8a, 0x9c9d, 0x9bb5, 0x9ad1,
    0x99f0, 0x9913, 0x983a, 0x9765, 0x9693, 0x95c4, 0x94f8, 0x9430, 0x936b, 0x92a9, 0x91ea, 0x912e,
    0x9075, 0x8fbe, 0x8f0a, 0x8e59, 0x8daa, 0x8cfe, 0x8c54, 0x8bac, 0x8b07, 0x8a64, 0x89c4, 0x8925,
    0x8889, 0x87ee, 0x8756, 0x86c0, 0x862b, 0x8599, 0x8508, 0x8479, 0x83ec, 0x8361, 0x82d8, 0x8250,
    0x81c9, 0x8145, 0x80c2, 0x8040, 0xff02, 0xfd0e, 0xfb25, 0xf947, 0xf773, 0xf5aa, 0xf3ea, 0xf234,
    0xf087, 0xeee3, 0xed47, 0xebb3, 0xea27, 0xe8a3, 0xe727, 0xe5b2, 0xe443, 0xe2dc, 0xe17a, 0xe020,
    0xdecb, 0xdd7d, 0xdc34, 0xdaf1, 0xd9b3, 0xd87b, 0xd748, 0xd61a, 0xd4f1, 0xd3cd, 0xd2ad, 0xd192,
    0xd07b, 0xcf69, 0xce5b, 0xcd51, 0xcc4a, 0xcb48, 0xca4a, 0xc94f, 0xc858, 0xc764, 0xc674, 0xc587,
    0xc49d, 0xc3b7, 0xc2d4, 0xc1f4, 0xc116, 0xc03c, 0xbf65, 0xbe90, 0xbdbe, 0xbcef, 0xbc23, 0xbb59,
    0xba91, 0xb9cc, 0xb90a, 0xb84a, 0xb78c, 0xb6d0, 0xb617, 0xb560,
];

/* returns a*b*2^-32 - e, with error 0 <= e < 1.  */
fn mul32<const N: usize>(a: Simd<u32, N>, b: Simd<u32, N>) -> Simd<u32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    ((a.cast::<u64>() * b.cast::<u64>()) >> 32).cast()
}

/* returns a*b*2^-64 - e, with error 0 <= e < 3.  */
/*
fn mul64(a: u64, b: u64) -> u64 {
    let ahi = a >> 32;
    let alo = a & 0xffffffff;
    let bhi = a >> 32;
    let blo = b & 0xffffffff;
    ahi * bhi + (ahi * blo >> 32) + (alo * bhi >> 32)
}
*/

macro_rules! make_f32_fns {
    { $($ty:ident, $len:literal)* } => {
        $(
        paste::paste! {
            #[no_mangle]
            pub fn [<vapor_sqrt_ $ty>](x: $ty) -> $ty {
                vectorize!($len, {
                    if (x == scalar!(f32::INFINITY)) | (x == 0.0) {
                        x
                    } else if x.is_nan() | (x < 0.0) {
                        scalar!(f32::NAN)
                    } else {
                        let x1p23 = scalar!(f32::from_bits(0x4b000000));
                        let x = if x.is_subnormal() {
                            <f32>::from_bits((x * x1p23).to_bits() - (23u32 << 23))
                        } else {
                            x
                        };

                        let even = x.to_bits() & 0x00800000 != 0;
                        let m = if even {
                            (x.to_bits() << 7) & 0x7fffffff
                        } else {
                            (x.to_bits() << 8) | 0x80000000
                        };

                        let mut ey = x.to_bits() >> 1;
                        ey += 0x3f800000u32 >> 1;
                        ey &= 0x7f800000;

                        let three = 0xc0000000;
                        let i = (x.to_bits() >> 17) % 128;
                        let mut r = <u32>::gather_or(&RSQRT_TAB, i as usize, 0) << 16;
                        let mut s = mul32(m, r);
                        let mut d = mul32(s, r);
                        let mut u = three - d;
                        r = mul32(r, u) << 1;
                        s = mul32(s, u) << 1;
                        d = mul32(s, r);
                        u = three - d;
                        s = mul32(s, u);
                        s = (s - 1) >> 6;

                        let d0 = (m << 16) - s * s;
                        let d1 = s - d0;
                        let d2 = d1 + s + 1;
                        s += d1 >> 31;
                        s &= 0x007fffff;
                        s |= ey;
                        let y = <f32>::from_bits(s);

                        let mut tiny = if d2 == 0 { 0 } else { 0x01000000 };
                        tiny |= (d1 ^ d2) & 0x80000000;
                        y + <f32>::from_bits(tiny)
                    }
                })
            }
        }
        )*
    }
}

make_f32_fns! {
    f32x2, 2
    f32x4, 4
    f32x8, 8
}
