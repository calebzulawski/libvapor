---cargo
[package]
edition = "2021"
---

// Compile this probe for baseline x86-64 so detection happens at runtime.
#[cfg(target_arch = "x86_64")]
fn main() {
    use std::arch::x86_64::__cpuid;

    macro_rules! detected {
        ($($feature:tt),+) => { true $(&& std::is_x86_feature_detected!($feature))+ };
    }

    // LAHF/SAHF has no is_x86_feature_detected! entry. Check its CPUID bit.
    // Query the maximum extended leaf before reading the feature bit.
    let lahf_sahf = __cpuid(0x80000000).eax >= 0x80000001 && __cpuid(0x80000001).ecx & 1 != 0;
    let v2 = lahf_sahf && detected!("cmpxchg16b", "popcnt", "sse3", "ssse3", "sse4.1", "sse4.2");
    // Rust's AVX detection also checks the OS's enabled register state.
    let v3 =
        v2 && detected!("avx", "avx2", "bmi1", "bmi2", "f16c", "fma", "lzcnt", "movbe", "xsave");
    let v4 = v3 && detected!("avx512f", "avx512bw", "avx512cd", "avx512dq", "avx512vl");
    println!(
        "{}",
        if v4 {
            4
        } else if v3 {
            3
        } else if v2 {
            2
        } else {
            1
        }
    );
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("The level comparison requires an x86-64 host.");
    std::process::exit(1);
}
