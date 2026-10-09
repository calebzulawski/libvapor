# libm implementation status (f32/f64)

✅ Implemented · ❌ Not implemented · 🚫 Not planned

## [C23: Mathematics (<math.h>), section 7.12](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf)

### 7.12.3 Classification macros

| Status | Macro | Note |
| --- | --- | --- |
| ✅ | `fpclassify` | |
| 🚫 | `iscanonical` | Always true for f32/f64. |
| ✅ | `isfinite` | |
| ✅ | `isinf` | |
| ✅ | `isnan` | |
| ✅ | `isnormal` | |
| ✅ | `signbit` | |
| ✅ | `issignaling` | |
| ✅ | `issubnormal` | |
| ✅ | `iszero` | |

### 7.12.4 Trigonometric functions

| Status | Function |
| --- | --- |
| ✅ | `acos` |
| ✅ | `asin` |
| ✅ | `atan` |
| ✅ | `atan2` |
| ✅ | `cos` |
| ✅ | `sin` |
| ✅ | `tan` |
| ❌ | `acospi` |
| ❌ | `asinpi` |
| ❌ | `atanpi` |
| ❌ | `atan2pi` |
| ❌ | `cospi` |
| ❌ | `sinpi` |
| ❌ | `tanpi` |

### 7.12.5 Hyperbolic functions

| Status | Function |
| --- | --- |
| ✅ | `acosh` |
| ✅ | `asinh` |
| ✅ | `atanh` |
| ✅ | `cosh` |
| ✅ | `sinh` |
| ✅ | `tanh` |

### 7.12.6 Exponential and logarithmic functions

| Status | Function |
| --- | --- |
| ✅ | `exp` |
| ❌ | `exp10` |
| ❌ | `exp10m1` |
| ✅ | `exp2` |
| ❌ | `exp2m1` |
| ✅ | `expm1` |
| ❌ | `frexp` |
| ❌ | `ilogb` |
| ❌ | `ldexp` |
| ❌ | `llogb` |
| ✅ | `log` |
| ✅ | `log10` |
| ❌ | `log10p1` |
| ✅ | `log1p` |
| ✅ | `logp1` |
| ✅ | `log2` |
| ❌ | `log2p1` |
| ❌ | `logb` |
| ❌ | `modf` |
| ❌ | `scalbn` |
| ❌ | `scalbln` |

### 7.12.7 Power and absolute-value functions

| Status | Function |
| --- | --- |
| ✅ | `cbrt` |
| ❌ | `compoundn` |
| ✅ | `fabs` |
| ✅ | `hypot` |
| ✅ | `pow` |
| ❌ | `pown` |
| ❌ | `powr` |
| ❌ | `rootn` |
| ❌ | `rsqrt` |
| ✅ | `sqrt` |

### 7.12.8 Error and gamma functions

| Status | Function |
| --- | --- |
| ✅ | `erf` |
| ✅ | `erfc` |
| ✅ | `lgamma` |
| ✅ | `tgamma` |

### 7.12.9 Nearest integer functions

| Status | Function | Note |
| --- | --- | --- |
| ✅ | `ceil` | |
| ✅ | `floor` | |
| 🚫 | `nearbyint` | Reads the current rounding mode. |
| 🚫 | `rint` | Reads the current rounding mode. |
| 🚫 | `lrint` | Reads the current rounding mode. |
| 🚫 | `llrint` | Reads the current rounding mode. |
| ✅ | `round` | |
| ❌ | `lround` | |
| ❌ | `llround` | |
| ❌ | `roundeven` | |
| ✅ | `trunc` | |
| 🚫 | `fromfp` | Controls floating-point exception flags. |
| 🚫 | `ufromfp` | Controls floating-point exception flags. |
| 🚫 | `fromfpx` | Controls floating-point exception flags. |
| 🚫 | `ufromfpx` | Controls floating-point exception flags. |

### 7.12.10 Remainder functions

| Status | Function |
| --- | --- |
| ✅ | `fmod` |
| ✅ | `remainder` |
| ❌ | `remquo` |

### 7.12.11 Manipulation functions

| Status | Function | Note |
| --- | --- | --- |
| ✅ | `copysign` | |
| 🚫 | `nan` | Parses a string. |
| ❌ | `nextafter` | |
| 🚫 | `nexttoward` | Requires `long double`. |
| ❌ | `nextup` | |
| ❌ | `nextdown` | |
| ❌ | `canonicalize` | |

### 7.12.12 Maximum, minimum, and positive difference functions

| Status | Function |
| --- | --- |
| ❌ | `fdim` |
| ✅ | `fmax` |
| ✅ | `fmin` |
| ❌ | `fmaximum` |
| ❌ | `fminimum` |
| ❌ | `fmaximum_mag` |
| ❌ | `fminimum_mag` |
| ❌ | `fmaximum_num` |
| ❌ | `fminimum_num` |
| ❌ | `fmaximum_mag_num` |
| ❌ | `fminimum_mag_num` |

### 7.12.13 Fused multiply-add

| Status | Function |
| --- | --- |
| ✅ | `fma` |

### 7.12.14 Functions that round result to narrower type

| Status | Function |
| --- | --- |
| ❌ | `fadd` |
| ❌ | `fsub` |
| ❌ | `fmul` |
| ❌ | `fdiv` |
| ❌ | `ffma` |
| ❌ | `fsqrt` |

### 7.12.17 Comparison macros

| Status | Macro | Note |
| --- | --- | --- |
| ✅ | `isgreater` | |
| ✅ | `isgreaterequal` | |
| ✅ | `isless` | |
| ✅ | `islessequal` | |
| ✅ | `islessgreater` | |
| ✅ | `isunordered` | |
| 🚫 | `iseqsig` | Sets math error state for NaNs. |

## [C23: ISO/IEC 60559 floating-point arithmetic, Annex F (conditional)](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf#page=524)

### F.10.12 Total order functions

| Status | Function |
| --- | --- |
| ✅ | `totalorder` |
| ✅ | `totalordermag` |

### F.10.13 Payload functions

| Status | Function |
| --- | --- |
| ❌ | `getpayload` |
| ❌ | `setpayload` |
| ❌ | `setpayloadsig` |

## Extensions

| Status | Function |
| --- | --- |
| ✅ | `sincos` |
