//! Portable SIMD logarithms, assuming round-to-nearest, ties-to-even.
//!
//! Natural and binary logarithms share input normalization and table reduction.
//! Decimal and one-plus logarithms share a series kernel on a reduced interval.

mod binary;
mod data;
mod decimal;
mod natural;
mod one_plus;
mod reduction;
mod series;

pub use binary::*;
pub use decimal::*;
pub use natural::*;
pub use one_plus::*;
