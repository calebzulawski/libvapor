use libvapor_tools::oracle::PRECISION;
use rug::Float;

fn main() {
    let source = include_str!("../../../src/log/table_fma/log.rs");
    let table = source.split_once("const INVC:").unwrap().1;
    let table = table.split_once("];").unwrap().0;
    let reciprocal: Vec<_> = table
        .lines()
        .filter_map(|line| {
            let hex = line.trim().strip_prefix("f64::from_bits(0x")?;
            let hex = hex.split(')').next().unwrap();
            Some(f64::from_bits(u64::from_str_radix(hex, 16).unwrap()))
        })
        .collect();
    assert_eq!(reciprocal.len(), 128);
    let centers: Vec<_> = reciprocal
        .into_iter()
        .map(|invc| {
            let center = Float::with_val(PRECISION, invc).recip();
            let hi = center.to_f64();
            let lo = Float::with_val(PRECISION, center - hi).to_f64();
            [hi, lo]
        })
        .collect();

    print!("{}", source.split("use core::simd").next().unwrap());
    println!("// Generated split reciprocals of the existing INVC table.");
    println!("// Regenerate with: cargo run -p libvapor-tools --features oracle \\");
    println!("//   --bin generate-log-centers > src/log/table_non_fma/centers.rs\n");
    for (field, name) in ["CENTER_HI", "CENTER_LO"].into_iter().enumerate() {
        println!("pub(super) const {name}: [f64; 128] = [");
        for center in &centers {
            println!("    f64::from_bits(0x{:016x}),", center[field].to_bits());
        }
        println!("];");
        if field == 0 {
            println!();
        }
    }
}
