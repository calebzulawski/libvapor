use super::*;
use proptest::{
    prelude::*,
    test_runner::{Config, TestRunner},
};

fn check(cases: &[[f64; 3]]) {
    fn run<const N: usize>(cases: &[[f64; 3]]) {
        for batch in cases.chunks(N) {
            let [x, y, z] = std::array::from_fn(|column| {
                Simd::<f64, N>::from_array(std::array::from_fn(|lane| {
                    batch[lane % batch.len()][column]
                }))
            });
            for got in [
                integer::fma_f64(x, y, z),
                double_double::fma_f64(x, y, z),
                crate::fma_f64(x, y, z),
            ] {
                for (lane, got) in got.to_array().into_iter().enumerate() {
                    let [x, y, z] = batch[lane % batch.len()];
                    let expected = x.mul_add(y, z);
                    assert!(
                        if expected.is_nan() {
                            got.is_nan()
                        } else {
                            got.to_bits() == expected.to_bits()
                        },
                        "f64x{N} input {:x?}: got {got:?}, expected {expected:?}",
                        [x, y, z].map(f64::to_bits),
                    );
                }
            }
        }
    }

    run::<1>(cases);
    run::<2>(cases);
    run::<3>(cases);
    run::<4>(cases);
    run::<8>(cases);
    run::<16>(cases);
    run::<64>(cases);
}

#[test]
fn extreme_exponents_and_rounding() {
    let tiny = f64::from_bits(1);
    let min = f64::MIN_POSITIVE;
    let eps = f64::EPSILON;
    let mut cases = vec![
        [1.0 + eps, 1.0 - eps, -1.0],
        [f64::MAX, 2.0, -f64::MAX],
        [f64::MAX, 2.0, f64::NEG_INFINITY],
        [f64::MAX, f64::MAX, f64::NEG_INFINITY],
        [min, 0.5, -f64::from_bits(0x0008000000000000)],
        [min, 0.5, -f64::from_bits(0x0007ffffffffffff)],
        [min, 0.5, -f64::from_bits(0x0008000000000001)],
        [tiny, tiny, tiny],
        [tiny, tiny, -tiny],
        [tiny, 0.5, tiny],
        [tiny, 0.5, 2.0 * tiny],
        [tiny, 0.5, -tiny],
        [1.5, tiny, -tiny],
        [1.5 + eps, tiny, -tiny],
        [1.5 - eps, tiny, -tiny],
        [min, 1.0 - eps / 2.0, -tiny],
        [min, 1.0 - eps, tiny],
        [1.0 + eps, min / 2.0, min / 2.0],
        [1.0 - eps / 2.0, min / 2.0, min / 2.0],
        [1.0, eps / 2.0, 1.0],
        [1.0 + eps, eps / 2.0, 1.0],
        [1.0 - eps / 2.0, eps / 2.0, 1.0],
        [f64::MAX, 1.0, f64::from_bits(0x7c90000000000000)], // overflow midpoint
        [f64::MAX, 1.0, f64::from_bits(0x7c8fffffffffffff)],
    ];
    // A product near one has normalized exponent -106; an addend with
    // exponent delta - 53 gives exactly the desired alignment distance.
    for delta in [-65, -64, -63, -1, 0, 1, 63, 64, 65, 127, 128, 129] {
        let addend = f64::from_bits(((delta - 53 + 1023) as u64) << 52);
        cases.push([1.0 + eps, 1.0 + eps, addend]);
        cases.push([1.0 + eps, 1.0 + eps, -addend]);
    }
    // Cover the exponent range, including sticky bits lost across the
    // 64-bit words and exact or almost exact cancellation.
    for exponent in [
        -1074, -1022, -1021, -64, -63, -1, 0, 1, 63, 64, 65, 127, 128, 1023,
    ] {
        let scale = if exponent == -1074 {
            tiny
        } else {
            f64::from_bits(((exponent + 1023) as u64) << 52)
        };
        for addend in [tiny, min, 1.0, 1.0 + eps, f64::MAX] {
            cases.push([scale, 1.0 + eps, addend]);
            cases.push([scale, 1.0 + eps, -addend]);
        }
    }
    // Exhaust exceptional values and signed zeros in every argument position.
    let special = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    for x in special {
        for y in special {
            for z in special {
                cases.push([x, y, z]);
            }
        }
    }
    let positive_cases = cases.clone();
    for [x, y, z] in positive_cases {
        cases.push([-x, y, -z]);
        cases.push([x, -y, -z]);
        cases.push([-x, -y, z]);
    }
    check(&cases);
}

#[test]
fn double_double_range_and_split_boundaries() {
    let mut operands = vec![1.0, 1.0 + f64::EPSILON, 1.0 - f64::EPSILON];
    for exponent in [-450, 450] {
        let bits = ((exponent + 1023) as u64) << 52;
        for bits in [bits - 1, bits, bits + 1] {
            operands.push(f64::from_bits(bits));
        }
    }
    // Exercise either side of the 26-bit split's midpoints, including
    // a carry into the exponent. Random inputs rarely hit these bits.
    let midpoint = 1u64 << 26;
    let next_exponent = 1u64 << 52;
    for exponent in [-449, 0, 449] {
        let base = ((exponent + 1023) as u64) << 52;
        for fraction in [
            midpoint - 1,
            midpoint,
            midpoint + 1,
            3 * midpoint - 1,
            3 * midpoint,
            3 * midpoint + 1,
            next_exponent - midpoint - 1,
            next_exponent - midpoint,
            next_exponent - midpoint + 1,
        ] {
            operands.push(f64::from_bits(base | fraction));
        }
    }
    let mut addends = vec![1.0, -1.0];
    for exponent in [-900, 900] {
        let bits = ((exponent + 1023) as u64) << 52;
        for bits in [bits - 1, bits, bits + 1] {
            addends.extend([f64::from_bits(bits), -f64::from_bits(bits)]);
        }
    }
    let mut cases = vec![];
    for &x in &operands {
        for &y in &operands {
            for &z in &addends {
                cases.extend([[x, y, z], [-x, y, -z]]);
            }
            let product = x * y;
            // Exact or almost exact cancellation and halfway addends
            // expose the low product bits on either side of a split.
            for z in [
                product,
                -product,
                -f64::from_bits(product.to_bits() - 1),
                -f64::from_bits(product.to_bits() + 1),
                product * (f64::EPSILON / 2.0),
                -product * (f64::EPSILON / 2.0),
            ] {
                cases.extend([[x, y, z], [-x, y, -z], [x, -y, -z], [-x, -y, z]]);
            }
        }
    }
    check(&cases);
}

#[test]
fn double_double_random() {
    fn bounded(bits: u64, limit: u64) -> f64 {
        let exponent = 1023 - limit + ((bits >> 53) % (2 * limit + 1));
        f64::from_bits((bits & 0x800fffffffffffff) | (exponent << 52))
    }
    let values = (any::<u64>(), any::<u64>(), any::<u64>(), 0u8..3).prop_map(|(x, y, z, mode)| {
        let x = bounded(x, 400);
        let y = bounded(y, 400);
        let z = match mode {
            0 => bounded(z, 800),
            1 => -(x * y),
            _ => -f64::from_bits((x * y).to_bits() + 1),
        };
        [x, y, z]
    });
    let strategy = proptest::collection::vec(values, 64);
    let mut runner = TestRunner::new(Config::with_source_file(file!()));
    runner
        .run(&strategy, |cases| {
            check(&cases);
            Ok(())
        })
        .unwrap();
}
