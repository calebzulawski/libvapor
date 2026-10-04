//! Compile-time dispatch policy shared by the public entry points.
//! `force-soft` overrides target capabilities without disabling SIMD arithmetic.

pub(crate) const USE_HARDWARE_FMA: bool = cfg!(all(
    not(feature = "force-soft"),
    any(
        target_feature = "fma",
        all(target_arch = "aarch64", target_feature = "neon")
    )
));

pub(crate) const USE_HARDWARE_SQRT: bool = cfg!(all(
    not(feature = "force-soft"),
    any(
        target_feature = "sse2",
        all(target_arch = "aarch64", target_feature = "neon")
    )
));

pub(crate) const USE_HARDWARE_ROUND: bool = cfg!(all(
    not(feature = "force-soft"),
    any(
        target_feature = "sse4.1",
        all(target_arch = "aarch64", target_feature = "neon")
    )
));
