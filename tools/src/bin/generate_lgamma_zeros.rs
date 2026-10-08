//! Generate local Taylor expansions around negative zeros of lgamma.

use rug::{ops::Pow, Float};

// Near a pole, an exactly representable leading coefficient can leave a
// residual hundreds of digits smaller. Retain that residual before splitting.
const PRECISION: u32 = 1536;
const ORDER: u32 = 20;
const TERMS: u32 = 512;

fn logarithm(x: &Float) -> Float {
    x.clone().ln_abs_gamma().0
}

fn bisect(mut a: Float, mut b: Float) -> f64 {
    let mut fa = logarithm(&a);
    for _ in 0..440 {
        let mid = Float::with_val(PRECISION, &a + &b) / 2_u32;
        let fm = logarithm(&mid);
        if (fa < 0) == (fm < 0) {
            a = mid;
            fa = fm;
        } else {
            b = mid;
        }
    }
    (Float::with_val(PRECISION, a + b) / 2_u32).to_f64()
}

fn split(value: Float) -> [f64; 2] {
    let hi = value.to_f64();
    [hi, (value - hi).to_f64()]
}

fn hurwitz_zeta(k: u32, q: &Float, zeta_tail: &[Float]) -> Float {
    // For 1 <= q < 2, expand around 2:
    // zeta(k,q) = sum_j binomial(k+j-1,j) (2-q)^j (zeta(k+j)-1).
    // Each underlying geometric series has ratio at most 1/2. At these
    // orders, 512 terms leave less than 2^-400 absolute truncation error.
    let delta = Float::with_val(PRECISION, 2 - q);
    let mut weight = Float::with_val(PRECISION, 1);
    let mut sum = Float::with_val(PRECISION, 0);
    for j in 0..TERMS {
        sum += Float::with_val(PRECISION, &weight * &zeta_tail[(k + j - 2) as usize]);
        weight *= &delta;
        weight *= k + j;
        weight /= j + 1;
    }
    sum
}

fn coefficients(center: f64, scale: f64, zeta_tail: &[Float]) -> [[f64; 2]; 21] {
    let x = Float::with_val(PRECISION, center);
    let mut coefficients = [[0.0; 2]; 21];
    coefficients[0] = split(logarithm(&x));
    coefficients[1] = split(x.clone().digamma() * scale);

    // Shift the argument into [1,2). Differentiating
    // log|Gamma(x)| = log Gamma(x+n) - sum_i log|x+i| gives, for k >= 2,
    // coefficient[k] = (-1)^k/k * (scale^k*zeta(k,x+n)
    //                              + sum_i (scale/(x+i))^k).
    let mut q = x;
    let mut ratios = Vec::new();
    while q < 1 {
        ratios.push(Float::with_val(PRECISION, scale / &q));
        q += 1;
    }
    for k in 2..=ORDER {
        let mut value = hurwitz_zeta(k, &q, zeta_tail);
        value *= Float::with_val(PRECISION, scale).pow(k);
        for ratio in &ratios {
            value += Float::with_val(PRECISION, ratio.pow(k));
        }
        value /= k;
        if k % 2 != 0 {
            value = -value;
        }
        coefficients[k as usize] = split(value);
    }
    coefficients
}

fn main() {
    let zeta_tail: Vec<_> = (2..ORDER + TERMS)
        .map(|k| Float::with_val(PRECISION, Float::zeta_u(k)) - 1)
        .collect();

    println!("// Generated local Taylor expansions using {PRECISION}-bit MPFR arithmetic.");
    println!("// These constants are computed from mathematical definitions.");
    println!("// Regenerate with: cargo run -p mwise-tools --features oracle \\");
    println!("//   --bin generate-lgamma-zeros > src/gamma/zeros.rs");
    println!("pub struct Zero {{");
    println!("    pub center: f64,");
    println!("    pub scale: f64,");
    println!("    pub radius: f64,");
    println!("    pub coefficients: [[f64; 2]; 21],");
    println!("}}");
    println!("pub const ZEROS: &[Zero] = &[");
    let mut count = 0;
    for n in 2_i32..18 {
        let midpoint = Float::with_val(PRECISION, -n) - 0.5_f64;
        let epsilon = Float::with_val(PRECISION, 1) >> 350_u32;
        let left = Float::with_val(PRECISION, -(n + 1)) + &epsilon;
        let right = Float::with_val(PRECISION, -n) - &epsilon;
        for center in [bisect(left, midpoint.clone()), bisect(midpoint, right)] {
            if center.fract() == 0.0 {
                continue;
            }
            let distance = (center - center.round()).abs();
            // Keep only the exponent to obtain 2^floor(log2(distance)).
            let scale = f64::from_bits(distance.to_bits() & 0x7ff0_0000_0000_0000);
            let radius = distance / 32.0;
            println!("    Zero {{");
            println!("        center: {center:?},");
            println!("        scale: {scale:?},");
            println!("        radius: {radius:?},");
            println!("        coefficients: [");
            for [hi, lo] in coefficients(center, scale, &zeta_tail) {
                println!("            [{hi:?}, {lo:?}],");
            }
            println!("        ],");
            println!("    }},");
            count += 1;
        }
    }
    println!("];");
    eprintln!("Generated {count} local expansions");
}
