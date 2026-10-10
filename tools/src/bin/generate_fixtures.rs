use mwise_tools::{oracle, serialization, Width};
use std::path::PathBuf;
#[path = "../../../src/gamma/zeros.rs"]
#[allow(dead_code)]
mod gamma_zeros;

const NEW_OPERATIONS: &[&str] = &[
    "fmod",
    "remainder",
    "cbrt",
    "hypot",
    "expm1",
    "pow",
    "powr",
    "pown",
    "rootn",
    "compoundn",
    "rsqrt",
    "sinh",
    "cosh",
    "tanh",
    "asinh",
    "acosh",
    "atanh",
    "erf",
    "erfc",
    "lgamma",
    "tgamma",
];

fn new_inputs(width: Width, op: &str) -> Vec<[u64; 3]> {
    let minimum = match width {
        Width::F32 => f32::MIN_POSITIVE as f64,
        Width::F64 => f64::MIN_POSITIVE,
    };
    let maximum = match width {
        Width::F32 => f32::MAX as f64,
        Width::F64 => f64::MAX,
    };
    let values = [
        -maximum,
        -minimum,
        -0.0,
        0.0,
        minimum,
        maximum,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        -185.0,
        -180.0,
        -170.5,
        -16.0,
        -7.0,
        -3.0,
        -2.747682646727412,
        f64::from_bits(0xc005fbbe9738207d),
        -2.4570247382208006,
        -2.0,
        -1.5,
        -1.0,
        -0.5,
        0.5,
        0.84375,
        1.0,
        1.1,
        1.25,
        2.0,
        2.2,
        2.3,
        2.5,
        2.857142857142857,
        4.2,
        f64::from_bits(0x4011f6d82f48ffce),
        f32::from_bits(0x41089b48) as f64,
        6.0,
        7.0,
        19.0,
        20.0,
        27.3,
        28.0,
        37.5,
        88.72283905206835,
        89.41598629223294,
        171.6243769563027,
        f64::from_bits(0x406b5d688f4bffff),
        709.782712893384,
        710.4758600739439,
    ];
    let mut inputs = Vec::new();
    if matches!(op, "pow" | "powr") {
        // Exercise the compensated logarithm's interval boundaries with
        // exponents large enough to amplify reduction errors, including
        // bases adjacent to one and outputs near underflow and overflow.
        let targets: &[f64] = match width {
            Width::F32 => &[-103.0, -87.0, -0.35, 0.35, 88.0],
            Width::F64 => &[-744.0, -708.0, -0.35, 0.35, 709.0],
        };
        for i in 0..=96 {
            let center = width.bits(0.75 + i as f64 / 128.0);
            for offset in [-1, 0, 1] {
                let bits = match width {
                    Width::F32 => (center as u32).wrapping_add_signed(offset) as u64,
                    Width::F64 => center.wrapping_add_signed(offset as i64),
                };
                let x = width.value(bits);
                if x != 1.0 {
                    for &target in targets {
                        inputs.push([bits, width.bits(target / x.ln()), 0]);
                    }
                }
            }
        }
    }
    if matches!(op, "pown" | "compoundn") {
        // Small-exponent dispatch boundaries, including bases whose rounded
        // 1+x is on a boundary while its compensated low part is nonzero.
        for base in [1.0 / 65536.0, 1.0 / 128.0, 0.125, 1.0, 4.0, 128.0, 65536.0] {
            for sign in [-1.0, 1.0] {
                let x = if op == "pown" {
                    sign * base
                } else {
                    base - 1.0
                };
                let center = width.bits(x);
                for offset in [-1, 0, 1] {
                    let bits = match width {
                        Width::F32 => (center as u32).wrapping_add_signed(offset) as u64,
                        Width::F64 => center.wrapping_add_signed(offset as i64),
                    };
                    for n in [
                        -129_i64, -128, -127, -65, -64, -63, -33, -32, -31, -17, -16, -15, -1, 0,
                        1, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129,
                    ] {
                        inputs.push([bits, n as u64, 0]);
                    }
                }
            }
        }
        // Consecutive finite small-exponent cases exercise the multiplication
        // path with mixed exponents at every vector width in the fixture tests.
        let mut state = 0x243f6a8885a308d3_u64;
        for i in 0..1024 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let unit = (state >> 11) as f64 / (1_u64 << 53) as f64;
            let x = if op == "pown" {
                (0.03125 + 15.96875 * unit) * if i % 2 == 0 { 1.0 } else { -1.0 }
            } else {
                -0.875 + 4.875 * unit
            };
            inputs.push([width.bits(x), (i as i64 % 65 - 32) as u64, 0]);
            inputs.push([width.bits(x), (i as i64 % 257 - 128) as u64, 0]);
        }
    }
    if matches!(op, "pown" | "compoundn") {
        // Amplify low bits near one, including a 1+x that rounds to one.
        for power in [24, 53, 63, 64] {
            let delta = 2.0_f64.powi(-power);
            for x in [delta, -delta] {
                let x = if op == "pown" { 1.0 + x } else { x };
                let bits = width.bits(x);
                for n in [
                    i64::MIN,
                    i64::MAX,
                    (1_i64 << 62) + 1,
                    (1_i64 << 61) + 1,
                    (1_i64 << 53) + 1,
                    1000,
                    -1000,
                ] {
                    inputs.push([bits, n as u64, 0]);
                }
            }
        }
    }
    if op == "compoundn" {
        // Cancellation in the compensated logarithm must retain its denominator correction.
        for x in [
            f64::from_bits(0xbc6c3ee954000000),
            f64::from_bits(0xbc7de90f13ffffff),
        ] {
            inputs.push([width.bits(x), i64::MAX as u64, 0]);
        }
    }
    if matches!(op, "fmod" | "remainder") {
        // A rounded division can land on an integer or half-integer boundary.
        for bits in 1..=32 {
            let y = match width {
                Width::F32 => f32::from_bits(0x3f800000 + bits) as f64,
                Width::F64 => f64::from_bits(0x3ff0000000000000 + bits as u64),
            };
            let limit = match width {
                Width::F32 => 4194303.0,
                Width::F64 => 1125899906842623.0,
            };
            for q in [1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 65535.0, limit] {
                let x = width.bits(y * q);
                for offset in [-1, 0, 1] {
                    let x = match width {
                        Width::F32 => (x as u32).wrapping_add_signed(offset) as u64,
                        Width::F64 => x.wrapping_add_signed(offset as i64),
                    };
                    inputs.push([x, width.bits(y), 0]);
                }
            }
        }
    }
    if op == "lgamma" {
        for zero in gamma_zeros::ZEROS {
            for delta in [-zero.radius, 0.0, zero.radius] {
                let bits = width.bits(zero.center + delta);
                for offset in -8..=8 {
                    let bits = match width {
                        Width::F32 => (bits as u32).wrapping_add_signed(offset) as u64,
                        Width::F64 => bits.wrapping_add_signed(offset as i64),
                    };
                    inputs.push([bits, 0, 0]);
                }
            }
        }
    }
    for value in values {
        let bits = width.bits(value);
        for offset in [-1, 0, 1] {
            let bits = match width {
                Width::F32 => (bits as u32).wrapping_add_signed(offset) as u64,
                Width::F64 => bits.wrapping_add_signed(offset as i64),
            };
            if matches!(op, "pown" | "rootn" | "compoundn") {
                for n in [
                    i64::MIN,
                    i64::MAX,
                    -(1_i64 << 53) - 1,
                    (1_i64 << 53) + 1,
                    -1000,
                    -3,
                    -2,
                    -1,
                    0,
                    1,
                    2,
                    3,
                    1000,
                ] {
                    inputs.push([bits, n as u64, 0]);
                }
            } else if matches!(op, "pow" | "powr" | "hypot" | "fmod" | "remainder") {
                for y in [
                    -maximum,
                    -3.0,
                    -0.5,
                    -0.0,
                    0.0,
                    minimum,
                    0.5,
                    1.0,
                    2.0,
                    3.0,
                    maximum,
                    f64::INFINITY,
                    f64::NAN,
                ] {
                    inputs.push([bits, width.bits(y), 0]);
                }
                inputs.push([bits, 1, 0]);
                inputs.push([bits, 3, 0]);
            } else {
                inputs.push([bits, 0, 0]);
            }
        }
    }
    let mut state = 0x9e3779b97f4a7c15_u64;
    for i in 0..256 {
        let mut sample = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            if i % 2 == 0 {
                width.bits((state >> 11) as f64 / (1_u64 << 53) as f64 * 32.0 - 16.0)
            } else {
                match width {
                    Width::F32 => state as u32 as u64,
                    Width::F64 => state,
                }
            }
        };
        let x = sample();
        let y = if matches!(op, "pown" | "rootn" | "compoundn") {
            if i % 2 == 0 {
                (i as i64 % 33 - 16) as u64
            } else {
                sample()
            }
        } else if matches!(op, "pow" | "powr" | "hypot" | "fmod" | "remainder") {
            sample()
        } else {
            0
        };
        inputs.push([x, y, 0]);
    }
    inputs
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Keep the fixed inputs and their lane order. Fresh input sampling belongs
    // to the live property tests. An optional path writes a copy.
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(serialization::FIXTURE_PATH));
    let mut groups = serialization::load(serialization::FIXTURE_PATH)?;
    for width in [Width::F32, Width::F64] {
        for &op in NEW_OPERATIONS {
            if let Some(group) = groups
                .iter_mut()
                .find(|group| group.width == width && group.op == op)
            {
                for input in new_inputs(width, op) {
                    if !group.cases.iter().any(|case| case.input == input) {
                        group.cases.push(oracle::case(width, op, input));
                    }
                }
            } else {
                groups.push(serialization::Fixtures {
                    width,
                    op: op.into(),
                    cases: new_inputs(width, op)
                        .into_iter()
                        .map(|input| oracle::case(width, op, input))
                        .collect(),
                });
            }
        }
    }
    for group in &mut groups {
        for case in &mut group.cases {
            *case = oracle::case(group.width, &group.op, case.input);
        }
    }
    serialization::save(&path, &groups)?;
    let count: usize = groups.iter().map(|group| group.cases.len()).sum();
    println!(
        "Wrote {count} cases with {}-bit MPFR references to {}",
        oracle::PRECISION,
        path.display()
    );
    Ok(())
}
