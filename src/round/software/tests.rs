// Shared boundary coverage for the f32 and f64 software implementations.
// Instantiate beside each implementation with its conversion/precision limits.
macro_rules! rounding_boundaries {
    ($kind:ident; $($boundary:expr),+ $(,)?) => {
        #[test]
        fn rounding_boundaries() {
            fn check<const N: usize>() {
                let mut values = Vec::new();
                // All-small vectors exercise conversion at every lane count.
                // Random tests rarely hit the floats immediately around ties.
                for integer in 0..=64 {
                    let half = integer as $kind + 0.5;
                    for x in [half.next_down(), half, half.next_up()] {
                        values.extend([x, -x]);
                    }
                }
                for x in [$($boundary as $kind,)+ $kind::MAX] {
                    for x in [x.next_down(), x, x.next_up()] {
                        values.extend([x, -x]);
                    }
                }
                values.extend([
                    0.75,
                    -0.75,
                    0.0,
                    -0.0,
                    $kind::from_bits(1),
                    -$kind::from_bits(1),
                    $kind::MIN_POSITIVE,
                    -$kind::MIN_POSITIVE,
                    $kind::INFINITY,
                    $kind::NEG_INFINITY,
                    $kind::NAN,
                ]);
                for batch in values.chunks(N) {
                    let input: [$kind; N] = std::array::from_fn(|lane| batch[lane % batch.len()]);
                    let x = core::simd::Simd::from_array(input);
                    paste::paste! {
                        for (name, got, expected) in [
                            ("trunc", [<trunc_ $kind>](x).to_array(), input.map($kind::trunc)),
                            ("floor", [<floor_ $kind>](x).to_array(), input.map($kind::floor)),
                            ("ceil", [<ceil_ $kind>](x).to_array(), input.map($kind::ceil)),
                            ("round", [<round_ $kind>](x).to_array(), input.map($kind::round)),
                        ] {
                            for lane in 0..N {
                                assert!(if expected[lane].is_nan() { got[lane].is_nan() }
                                    else { got[lane].to_bits() == expected[lane].to_bits() },
                                    "{name} {}x{N}, input {:?}: got {:?}, expected {:?}",
                                    stringify!($kind), input[lane], got[lane], expected[lane]);
                            }
                        }
                    }
                }
            }
            check::<1>();
            check::<2>();
            check::<3>();
            check::<4>();
            check::<8>();
            check::<16>();
            check::<64>();
        }
    };
}

pub(super) use rounding_boundaries;
