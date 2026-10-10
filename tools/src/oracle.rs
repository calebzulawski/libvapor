//! MPFR references for transcendental tests.

use crate::{Case, Width};
use rug::ops::{PowAssignRound, RemAssignRound};
use rug::Assign;
use rug::{
    float::{Round, Special},
    Float,
};

pub const PRECISION: u32 = 768;

fn reference(op: &str, width: Width, input: [u64; 3], rounding: Round) -> Float {
    let mut x = Float::with_val(PRECISION, width.value(input[0]));
    let n = input[1] as i64;
    let y = if matches!(op, "pown" | "rootn" | "compoundn") {
        Float::with_val(PRECISION, n)
    } else {
        Float::with_val(PRECISION, width.value(input[1]))
    };
    match op {
        "fmod" => {
            x.rem_assign_round(&y, rounding);
        }
        "remainder" => {
            x.remainder_round(&y, rounding);
        }
        "cbrt" => {
            x.cbrt_round(rounding);
        }
        "hypot" => {
            x.hypot_round(&y, rounding);
        }
        "exp" => {
            x.exp_round(rounding);
        }
        "exp2" => {
            x.exp2_round(rounding);
        }
        "expm1" => {
            x.exp_m1_round(rounding);
        }
        "pow" | "pown" => {
            x.pow_assign_round(&y, rounding);
        }
        "powr" => {
            if x.is_nan()
                || y.is_nan()
                || x < 0
                || ((x.is_zero() || x.is_infinite()) && y.is_zero())
                || (x == 1 && y.is_infinite())
            {
                x.assign(Special::Nan);
            } else {
                // powr treats both signed zeros as a nonnegative base.
                x.abs_mut();
                x.pow_assign_round(&y, rounding);
            }
        }
        "rootn" => {
            if let Ok(n) = i32::try_from(n) {
                x.root_i_round(n, rounding);
            } else {
                let negative = x.is_sign_negative() && n & 1 != 0;
                if x < 0 && n & 1 == 0 {
                    x.assign(Special::Nan);
                } else {
                    x.abs_mut();
                    let exponent = Float::with_val(PRECISION, y.recip_ref());
                    // An odd negative root reverses the directed magnitude rounding.
                    let magnitude_round = if negative {
                        match rounding {
                            Round::Down => Round::Up,
                            Round::Up => Round::Down,
                            other => other,
                        }
                    } else {
                        rounding
                    };
                    x.pow_assign_round(&exponent, magnitude_round);
                    if negative {
                        x = -x;
                    }
                }
            }
        }
        "compoundn" => {
            if let Ok(n) = i32::try_from(n) {
                x.compound_i_round(n, rounding);
            } else if x < -1 {
                x.assign(Special::Nan);
            } else {
                x += 1;
                x.pow_assign_round(&y, rounding);
            }
        }
        "rsqrt" => {
            // MPFR returns +inf for -0; C23 preserves the zero's sign.
            if x.is_zero() && x.is_sign_negative() {
                x.assign(Special::NegInfinity);
            } else {
                x.recip_sqrt_round(rounding);
            }
        }
        "log" => {
            x.ln_round(rounding);
        }
        "log2" => {
            x.log2_round(rounding);
        }
        "log10" => {
            x.log10_round(rounding);
        }
        "log1p" => {
            x.ln_1p_round(rounding);
        }
        "sin" => {
            x.sin_round(rounding);
        }
        "cos" => {
            x.cos_round(rounding);
        }
        "tan" => {
            x.tan_round(rounding);
        }
        "asin" => {
            x.asin_round(rounding);
        }
        "acos" => {
            x.acos_round(rounding);
        }
        "atan" => {
            x.atan_round(rounding);
        }
        "atan2" => {
            x.atan2_round(&y, rounding);
        }
        "sinh" => {
            x.sinh_round(rounding);
        }
        "cosh" => {
            x.cosh_round(rounding);
        }
        "tanh" => {
            x.tanh_round(rounding);
        }
        "asinh" => {
            x.asinh_round(rounding);
        }
        "acosh" => {
            x.acosh_round(rounding);
        }
        "atanh" => {
            x.atanh_round(rounding);
        }
        "erf" => {
            x.erf_round(rounding);
        }
        "erfc" => {
            x.erfc_round(rounding);
        }
        "lgamma" => {
            x.ln_abs_gamma_round(rounding);
        }
        "tgamma" => {
            x.gamma_round(rounding);
        }
        _ => panic!("unknown operation: {op}"),
    }
    x
}

fn rounded(width: Width, value: &Float, rounding: Round) -> f64 {
    match width {
        Width::F32 => value.to_f32_round(rounding) as f64,
        Width::F64 => value.to_f64_round(rounding),
    }
}

fn bounds(width: Width, op: &str, input: [u64; 3]) -> [u64; 2] {
    let nearest = reference(op, width, input, Round::Nearest);
    let value = rounded(width, &nearest, Round::Nearest);
    if matches!(op, "fmod" | "remainder")
        || !value.is_finite()
        || nearest.is_zero()
        || (matches!(op, "sin" | "cos") && width.value(input[0]) == 0.0)
    {
        return [width.bits(value); 2];
    }

    let down = reference(op, width, input, Round::Down);
    let up = reference(op, width, input, Round::Up);
    let (precision, minimum_exponent) = match width {
        Width::F32 => (24, -149),
        Width::F64 => (53, -1074),
    };
    // A result just below a power of two has the smaller ULP spacing.
    // Directed references avoid doubling the tolerance through rounding.
    let exponent =
        (down.get_exp().unwrap().min(up.get_exp().unwrap()) - precision).max(minimum_exponent);
    let tolerance = Float::with_val(PRECISION, 4) << exponent;

    // Intersect both reference envelopes and round inward. Every accepted
    // float is within 4 ULP of the mathematical result, not its rounded value.
    let (lower, _) = Float::with_val_round(PRECISION, &up - &tolerance, Round::Up);
    let (upper, _) = Float::with_val_round(PRECISION, &down + &tolerance, Round::Down);
    let mut lower = rounded(width, &lower, Round::Up);
    let mut upper = rounded(width, &upper, Round::Down);
    if nearest.is_sign_positive() {
        lower = lower.max(0.0);
    } else {
        upper = upper.min(-0.0);
    }
    [width.bits(lower), width.bits(upper)]
}

pub fn case(width: Width, op: &str, input: [u64; 3]) -> Case {
    let names: &[&str] = if op == "sincos" {
        &["sin", "cos"]
    } else {
        &[op]
    };
    Case {
        input,
        bounds: names
            .iter()
            .map(|name| bounds(width, name, input))
            .collect(),
    }
}
