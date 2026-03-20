# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. FFI caller validity precondition for pow43 input domain (table index safety)

**Location**: `/data/test_case/lib.rs:1-188`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The original algorithm assumes (per comment) that callers only pass values for which the derived table indices are in-range and the arithmetic shift semantics match C expectations. In Rust, this is not enforced: pow43 accepts any i32 and must avoid panicking across FFI. The implementation compensates with runtime clamping via get(...).unwrap_or(&0.0), effectively defining a fallback behavior (0.0) for invalid/out-of-domain inputs rather than enforcing the input domain at compile time.

**Evidence**:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

static g_pow43: [f32; 145] = [
    0 as f32,
    -1_f32,
    -2.519842f32,
    -4.326749f32,
    -6.349604f32,
    -8.549_88_f32,
    -10.902724f32,
    -13.390518f32,
    -16.000000f32,
    -18.720754f32,
    -21.544347f32,
    -24.463781f32,
    -27.473142f32,
    -30.567_35_f32,
    -33.741992f32,
    -36.993_18_f32,
    0 as f32,
    1_f32,
    2.519842f32,
    4.326749f32,
    6.349604f32,
    8.549_88_f32,
    10.902724f32,
    13.390518f32,
    16.000000f32,
    18.720754f32,
    21.544347f32,
    24.463781f32,
    27.473142f32,
    30.567_35_f32,
    33.741992f32,
    36.993_18_f32,
    40.317474f32,
    43.711_79_f32,
    47.173345f32,
    50.699_63_f32,
    54.288352f32,
    57.937_41_f32,
    61.644865f32,
    65.408_94_f32,
    69.227_98_f32,
    73.100_44_f32,
    77.024898f32,
    81.000000f32,
    85.024_49_f32,
    89.097_19_f32,
    93.216_97_f32,
    97.382_8_f32,
    101.593667f32,
    105.848_63_f32,
    110.146801f32,
    114.487_32_f32,
    118.869381f32,
    123.292209f32,
    127.755065f32,
    132.257_25_f32,
    136.798_08_f32,
    141.376_9_f32,
    145.993_12_f32,
    150.646_12_f32,
    155.335_33_f32,
    160.060_2_f32,
    164.820_2_f32,
    169.614_82_f32,
    174.443_57_f32,
    179.305_98_f32,
    184.201_57_f32,
    189.129_91_f32,
    194.090_58_f32,
    199.083_15_f32,
    204.107_21_f32,
    209.162_38_f32,
    214.248_29_f32,
    219.364_56_f32,
    224.510_85_f32,
    229.686_78_f32,
    234.892_06_f32,
    240.126_33_f32,
    245.389_28_f32,
    250.680_6_f32,
    256.000000f32,
    261.347_17_f32,
    266.721_83_f32,
    272.123_72_f32,
    277.552_55_f32,
    283.008_06_f32,
    288.489_96_f32,
    293.998_05_f32,
    299.532_07_f32,
    305.091_77_f32,
    310.676_9_f32,
    316.287_26_f32,
    321.922_58_f32,
    327.582_7_f32,
    333.267_36_f32,
    338.976_38_f32,
    344.709_56_f32,
    350.466_64_f32,
    356.247_47_f32,
    362.051_88_f32,
    367.879_6_f32,
    373.730_53_f32,
    379.604_43_f32,
    385.501_13_f32,
    391.420_5_f32,
    397.362_3_f32,
    403.326_42_f32,
    409.312_68_f32,
    415.320_9_f32,
    421.350_9_f32,
    427.402_6_f32,
    433.475_74_f32,
    439.570_28_f32,
    445.685_97_f32,
    451.822_75_f32,
    457.980_44_f32,
    464.158_87_f32,
    470.357_97_f32,
    476.577_55_f32,
    482.817_44_f32,
    489.077_6_f32,
    495.357_88_f32,
    501.658_08_f32,
    507.978_15_f32,
    514.317_93_f32,
    520.677_3_f32,
    527.056_2_f32,
    533.454_4_f32,
    539.871_9_f32,
    546.308_5_f32,
    552.764_04_f32,
    559.238_6_f32,
    565.731_9_f32,
    572.243_9_f32,
    578.774_4_f32,
    585.323_5_f32,
    591.890_87_f32,
    598.476_56_f32,
    605.080_44_f32,
    611.702_33_f32,
    618.342_2_f32,
    625.000000f32,
    631.675_54_f32,
    638.368_8_f32,
    645.079_6_f32,
];

#[no_mangle]
pub extern "C" fn pow43(mut x: i32) -> f32 {
    // The original C implementation relies on arithmetic right shift and
    // out-of-range indexing being "impossible" for valid callers; however,
    // Rust must not panic across FFI. We preserve the math while ensuring
    // table access cannot panic for any i32 input.
    let mut mult: i32 = 256;

    // Fast path: direct table lookup for small x (including negatives).
    if x < 129 {
        let idx = 16i32.wrapping_add(x) as usize;
        // Avoid panic on invalid inputs; match C's "undefined" behavior by
        // returning 0.0 when the index would be out of bounds.
        return *g_pow43.get(idx).unwrap_or(&0.0f32);
    }

    if x < 1024 {
        mult = 16;
        x = x.wrapping_shl(3);
    }

    let sign: i32 = x.wrapping_mul(2) & 64;
    let denom_i: i32 = (x & !63).wrapping_add(sign);
    let frac: f32 = ((x & 63).wrapping_sub(sign)) as f32 / denom_i as f32;

    let base_idx_i: i32 = 16i32.wrapping_add((x.wrapping_add(sign)) >> 6);
    let base = *g_pow43.get(base_idx_i as usize).unwrap_or(&0.0f32);

    base * (1.0f32 + frac * (4.0f32 / 3.0f32 + frac * (2.0f32 / 9.0f32))) * mult as f32
}
```

**Entity:** pow43 (extern "C" fn) + g_pow43 table indexing

**States:** ValidCallerInputDomain, ArbitraryI32Input

**Transitions:**
- ArbitraryI32Input -> ValidCallerInputDomain via caller-side validation before calling pow43()

**Evidence:** comment in pow43: "out-of-range indexing being \"impossible\" for valid callers; however, Rust must not panic across FFI"; pow43 signature: `pub extern "C" fn pow43(mut x: i32) -> f32` accepts any i32 (no domain type); fast path index derivation: `let idx = 16i32.wrapping_add(x) as usize;` followed by `g_pow43.get(idx).unwrap_or(&0.0f32)`; second lookup index derivation: `let base_idx_i: i32 = 16i32.wrapping_add((x.wrapping_add(sign)) >> 6);` followed by `g_pow43.get(base_idx_i as usize).unwrap_or(&0.0f32)`

**Implementation:** Introduce a Rust-only safe wrapper that enforces the valid domain before reaching the lookup math, e.g., `struct Pow43Input(i32); impl TryFrom<i32> for Pow43Input { ...range checks... }` and `fn pow43_safe(x: Pow43Input) -> f32`. Keep `extern "C" fn pow43(i32)` as a thin adapter that validates/branches and then calls `pow43_safe` for valid values (or returns a defined error sentinel).

---

