//! Generate erf values and derivatives at exact dyadic centers.
use libvapor_tools::oracle::PRECISION;
use rug::{float::Constant, Float};

fn main() {
    println!("// Generated erf values and derivatives using {PRECISION}-bit MPFR arithmetic.");
    println!("// These constants are computed from mathematical definitions.");
    println!("// Regenerate with: cargo run -p libvapor-tools --features oracle \\");
    println!("//   --bin generate-erf-table > src/erf/centers.rs");
    println!("pub(super) const TABLE: [[f64; 2]; 769] = [");
    let mut factor = Float::with_val(PRECISION, Constant::Pi).sqrt().recip();
    factor *= 2;
    for i in 0..=768 {
        let center = Float::with_val(PRECISION, i as f64 / 128.0);
        let value = center.clone().erf().to_f64();
        let derivative = (-Float::with_val(PRECISION, &center * &center)).exp() * &factor;
        println!(
            "    [f64::from_bits(0x{:016x}), f64::from_bits(0x{:016x})],",
            value.to_bits(),
            derivative.to_f64().to_bits()
        );
    }
    println!("];");
}
