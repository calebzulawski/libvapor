//! Generate the compact compensated-log table from mathematical definitions.

use mwise_tools::oracle::PRECISION;
use rug::Float;

fn main() {
    println!("// Generated reciprocals and logarithms using {PRECISION}-bit MPFR arithmetic.");
    println!("// Regenerate with: cargo run -p mwise-tools --features oracle \\");
    println!("//   --bin generate-compensated-log > src/precision/log_table.rs");
    println!("// Each row holds the logarithm high bits and low bits, with the");
    println!("// reciprocal numerator packed into the low eight bits of the latter.");
    println!("pub const TABLE: [[u64; 2]; 96] = [");
    let mut maximum = 0.0_f64;
    for i in 0..96_u64 {
        // The interval midpoint is (193 + 2*i)/256. Round 128/midpoint
        // to the nearest integer; its odd denominator rules out ties.
        let denominator = 193 + 2 * i;
        let mut reciprocal = (32768 + denominator / 2) / denominator;
        // Both intervals touching one must reduce around one exactly,
        // preserving tiny logarithms multiplied by huge power exponents.
        if matches!(i, 31 | 32) {
            reciprocal = 128;
        }
        let logarithm = if reciprocal == 128 {
            Float::with_val(PRECISION, 0)
        } else {
            let inverse = Float::with_val(PRECISION, reciprocal as f64 / 128.0);
            -inverse.ln()
        };
        let hi = logarithm.to_f64();
        let lo = (logarithm - hi).to_f64();
        // Discarding eight low mantissa bits leaves about 98 bits overall.
        let packed = (lo.to_bits() & !255) | reciprocal;
        println!("    [0x{:016x}, 0x{packed:016x}],", hi.to_bits());
        for endpoint in [96 + i, 97 + i] {
            let reduced = endpoint as f64 * reciprocal as f64 / 16384.0 - 1.0;
            maximum = maximum.max(reduced.abs());
        }
    }
    println!("];");
    eprintln!("Maximum reduced argument magnitude: {maximum}");
}
