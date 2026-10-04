//! Shared trigonometric kernels, polynomial data, and full-range reduction.
//! Public functions and feature selection live in the operation modules.
pub(crate) mod data;
pub(crate) mod inverse_fma;
pub(crate) mod inverse_non_fma;
pub(crate) mod reduction;
pub(crate) mod sin_cos_fma;
pub(crate) mod sin_cos_non_fma;
pub(crate) mod sin_cos_non_fma_fast;
pub(crate) mod sin_cos_pair_fma;
pub(crate) mod sin_cos_pair_non_fma;
pub(crate) mod tan;
pub(crate) mod tan_fma;
