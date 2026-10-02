//! Portable SIMD trigonometric functions.
//!
//! Sine, cosine, and tangent share angle reduction. Arctangent is also the
//! basis for atan2; asin and acos share a rational approximation and sqrt.

mod asin_acos;
mod atan;
mod atan2;
mod data;
mod reduction;
mod sincos;
mod tan;

pub use asin_acos::*;
pub use atan::*;
pub use atan2::*;
pub use sincos::*;
pub use tan::*;
