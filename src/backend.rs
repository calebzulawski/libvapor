//! Compile-time dispatch policy shared by the public entry points.
//! `force-soft` overrides target capabilities without disabling SIMD arithmetic.

// Shared by public FMA and the FMA-based log, exp and trig algorithms.
#[inline]
pub(crate) const fn use_hardware_fma_f32<const N: usize>() -> bool {
    if cfg!(feature = "force-soft") {
        return false;
    }
    if cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "fma"
        ),
        all(target_arch = "aarch64", target_feature = "neon"),
        all(
            any(
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "loongarch64"
            ),
            target_feature = "f"
        ),
        // Earlier AltiVec f32 arithmetic may flush subnormals.
        all(
            any(target_arch = "powerpc", target_arch = "powerpc64"),
            target_feature = "vsx",
            target_feature = "power8-vector"
        ),
        all(target_arch = "s390x", target_feature = "vector"),
        // MSA provides FMA but LLVM doesn't reliably lower to it.
        // all(
        //     any(
        //         target_arch = "mips",
        //         target_arch = "mips64",
        //         target_arch = "mips32r6",
        //         target_arch = "mips64r6"
        //     ),
        //     target_feature = "msa",
        //     target_feature = "fp64"
        // ),
    )) {
        return true;
    }
    false
}

#[inline]
pub(crate) const fn use_hardware_fma_f64<const N: usize>() -> bool {
    if cfg!(feature = "force-soft") {
        return false;
    }
    if cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "fma"
        ),
        all(target_arch = "aarch64", target_feature = "neon"),
        all(
            any(
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "loongarch64"
            ),
            target_feature = "d"
        ),
        all(
            any(target_arch = "powerpc", target_arch = "powerpc64"),
            target_feature = "vsx"
        ),
        all(target_arch = "s390x", target_feature = "vector"),
        // MSA provides FMA but LLVM doesn't reliably lower to it.
        // all(
        //     any(
        //         target_arch = "mips",
        //         target_arch = "mips64",
        //         target_arch = "mips32r6",
        //         target_arch = "mips64r6"
        //     ),
        //     target_feature = "msa",
        //     target_feature = "fp64"
        // ),
    )) {
        return true;
    }
    false
}

#[inline]
pub(crate) const fn use_hardware_sqrt_f32<const N: usize>() -> bool {
    if cfg!(feature = "force-soft") {
        return false;
    }
    cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        ),
        all(target_arch = "aarch64", target_feature = "neon"),
        all(
            any(target_arch = "powerpc", target_arch = "powerpc64"),
            target_feature = "vsx"
        ),
        all(target_arch = "s390x", not(target_abi = "softfloat")),
        all(target_arch = "sparc64", target_feature = "v9"),
        all(
            any(
                target_arch = "mips",
                target_arch = "mips64",
                target_arch = "mips32r6",
                target_arch = "mips64r6"
            ),
            target_feature = "fp64"
        ),
        target_arch = "wasm32",
        target_arch = "wasm64",
        target_arch = "nvptx64",
        all(target_arch = "arm", target_feature = "vfp2sp"),
        all(
            any(
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "loongarch64"
            ),
            target_feature = "f"
        )
    ))
}

#[inline]
pub(crate) const fn use_hardware_sqrt_f64<const N: usize>() -> bool {
    if cfg!(feature = "force-soft") {
        return false;
    }
    cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        ),
        all(target_arch = "aarch64", target_feature = "neon"),
        all(
            any(target_arch = "powerpc", target_arch = "powerpc64"),
            target_feature = "vsx"
        ),
        all(target_arch = "s390x", not(target_abi = "softfloat")),
        all(target_arch = "sparc64", target_feature = "v9"),
        all(
            any(
                target_arch = "mips",
                target_arch = "mips64",
                target_arch = "mips32r6",
                target_arch = "mips64r6"
            ),
            target_feature = "fp64"
        ),
        target_arch = "wasm32",
        target_arch = "wasm64",
        target_arch = "nvptx64",
        all(
            target_arch = "arm",
            target_feature = "vfp2",
            target_feature = "fp64"
        ),
        all(
            any(
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "loongarch64"
            ),
            target_feature = "d"
        )
    ))
}

pub(crate) enum RoundingMode {
    Directed,
    TiesAway,
    TiesEven,
}

#[inline]
pub(crate) const fn use_hardware_round_f32<const N: usize>(mode: RoundingMode) -> bool {
    if cfg!(feature = "force-soft") {
        return false;
    }
    if cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1"
        ),
        all(target_arch = "aarch64", target_feature = "neon"),
        all(target_arch = "arm", target_feature = "fp-armv8"),
        all(
            any(target_arch = "powerpc", target_arch = "powerpc64"),
            target_feature = "vsx"
        ),
        all(target_arch = "s390x", target_feature = "vector"),
        all(
            any(target_arch = "riscv32", target_arch = "riscv64"),
            target_feature = "f"
        ),
        target_arch = "nvptx64"
    )) {
        return true;
    }
    // Wasm and LSX avoid library helpers for directed and ties-even rounding.
    // LLVM currently scalarizes LSX ties-even to frint.s/frint.d per lane.
    // Ties-away still calls a library helper, so keep it on our software path.
    matches!(mode, RoundingMode::Directed | RoundingMode::TiesEven)
        && cfg!(any(
            target_arch = "wasm32",
            target_arch = "wasm64",
            all(target_arch = "loongarch64", target_feature = "lsx")
        ))
}

#[inline]
pub(crate) const fn use_hardware_round_f64<const N: usize>(mode: RoundingMode) -> bool {
    if cfg!(feature = "force-soft") {
        return false;
    }
    if cfg!(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1"
        ),
        all(target_arch = "aarch64", target_feature = "neon"),
        all(
            target_arch = "arm",
            target_feature = "fp-armv8",
            target_feature = "fp64"
        ),
        all(
            any(target_arch = "powerpc", target_arch = "powerpc64"),
            target_feature = "vsx"
        ),
        all(target_arch = "s390x", target_feature = "vector"),
        all(target_arch = "riscv64", target_feature = "d"),
        // RV32 needs Zfa to round f64 without library calls.
        all(
            target_arch = "riscv32",
            target_feature = "d",
            target_feature = "zfa"
        ),
        target_arch = "nvptx64"
    )) {
        return true;
    }
    // Wasm and LSX avoid library helpers for directed and ties-even rounding.
    // LLVM currently scalarizes LSX ties-even to frint.s/frint.d per lane.
    // Ties-away still calls a library helper, so keep it on our software path.
    matches!(mode, RoundingMode::Directed | RoundingMode::TiesEven)
        && cfg!(any(
            target_arch = "wasm32",
            target_arch = "wasm64",
            all(target_arch = "loongarch64", target_feature = "lsx")
        ))
}
