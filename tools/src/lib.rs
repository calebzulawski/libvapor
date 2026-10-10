//! Shared helpers for fixture and live transcendental tests.

#[cfg(feature = "oracle")]
pub mod oracle;

pub mod serialization;
pub use serialization::{Case, Width};

pub fn check_bounds(width: Width, op: &str, output: usize, lane: usize, got: f64, case: &Case) {
    let [lower, upper] = case.bounds[output].map(|bits| width.value(bits));
    let context = || {
        format!("{width:?} {op} output {output} lane {lane}, input {:x?}: got {got:?}, expected [{lower:?}, {upper:?}]", case.input)
    };
    if lower.is_nan() {
        assert!(got.is_nan(), "{}", context());
    } else if lower.to_bits() == upper.to_bits() {
        assert_eq!(got.to_bits(), lower.to_bits(), "{}", context());
    } else {
        assert!(lower <= got && got <= upper, "{}", context());
        if got == 0.0 {
            assert_eq!(
                got.is_sign_negative(),
                lower.is_sign_negative(),
                "{}",
                context()
            );
        }
    }
}

/// Check a supplied vector function against reference bounds at one lane size.
#[macro_export]
macro_rules! check_accuracy {
    ($kind:ident, $width:ident, $lanes:expr, $cases:expr, $function:path,
        ($($arg:ident $( : $arg_type:ty)?),+) => $outputs:expr) => {{
        let cases = $cases;
        assert!(!cases.is_empty(), "{}: no reference cases", stringify!($function));
        for batch in cases.chunks($lanes) {
            let mut columns = 0..;
            $(let $arg = {
                let column = columns.next().unwrap();
                std::simd::Simd::<_, { $lanes }>::from_array(std::array::from_fn(|lane| {
                    $crate::check_accuracy!(@input $kind; batch[lane % batch.len()].input[column] $(, $arg_type)?)
                }))
            };)+
            let outputs = ($outputs)($function($($arg),+)).map(std::simd::Simd::to_array);
            // Partial vectors repeat valid cases; every lane is checked.
            for lane in 0..$lanes {
                let case = &batch[lane % batch.len()];
                assert_eq!(case.bounds.len(), outputs.len(), "{}: output count", stringify!($function));
                for (output, values) in outputs.iter().enumerate() {
                    $crate::check_bounds($crate::Width::$width,
                        stringify!($function), output, lane, values[lane] as f64, case);
                }
            }
        }
    }};
    (@input $kind:ident; $bits:expr) => {
        $kind::from_bits($bits.try_into().unwrap())
    };
    (@input $kind:ident; $bits:expr, $arg_type:ty) => {
        $bits as $arg_type
    };
}
