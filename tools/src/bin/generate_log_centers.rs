use mwise_tools::oracle::PRECISION;
use rug::Float;

#[path = "../../../src/log/table_non_fma/params.rs"]
mod params;
use params::{INDEX_SHIFT, OFFSET, TABLE_SIZE};

fn main() {
    println!("// Generated reciprocals and logarithms of exact dyadic centers.");
    println!("// Regenerate with: cargo run -p mwise-tools --features oracle \\");
    println!("//   --bin generate-log-centers > src/log/table_non_fma/centers.rs\n");
    for (base2, name) in [(false, "LOG_TABLE"), (true, "LOG2_TABLE")] {
        println!("pub(super) const {name}: [[f64; 2]; super::params::TABLE_SIZE] = [");
        for i in 0..TABLE_SIZE {
            let center = f64::from_bits(OFFSET + ((i as u64) << INDEX_SHIFT));
            let center = Float::with_val(PRECISION, center);
            let invc = center.clone().recip().to_f64();
            let logc = if base2 { center.log2() } else { center.ln() }.to_f64();
            println!("    [");
            println!("        f64::from_bits(0x{:016x}),", invc.to_bits());
            println!("        f64::from_bits(0x{:016x}),", logc.to_bits());
            println!("    ],");
        }
        println!("];");
        if !base2 {
            println!();
        }
    }
}
