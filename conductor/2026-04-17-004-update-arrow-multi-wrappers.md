# Implementation Plan - Update Arrow Multi Wrappers

Update `kand_arrow_wrapper_multi!` macro calls across the remaining files to match the new signature and ensure consistency with underlying raw functions.

## Context
The `kand_arrow_wrapper_multi!` macro has been updated to require explicit output types and return types, as well as lookback parameters for validation.

New Signature:
```rust
kand_arrow_wrapper_multi!(
    name_arrow,
    raw_fn,
    inputs: { ... },
    params: { ... },
    lookback_params: { ... },
    outputs: { name: type, ... },
    return_type: { Type, ... }
);
```

## Proposed Changes

### 1. `kand/src/ta/ohlcv/macd.rs`
- **Task**: Update `macd_raw` to only have 3 outputs (macd, signal, hist) and update arrow wrapper.
- **Details**:
    - Modify `macd_raw` and `macd` signatures.
    - Update implementation of `macd_raw` to use existing buffers for intermediate EMA calculations.
    - Update documentation and examples.
    - Update macro call.

### 2. `kand/src/ta/ohlcv/cdl_hammer.rs`
- **Task**: Update `cdl_hammer_arrow` macro call.
- **Signature**:
    - `outputs: { output_signals: TAInt, output_body_avg: TAFloat }`
    - `return_type: { TAArrowIntArray, TAArrowArray }`

### 3. `kand/src/ta/ohlcv/cdl_inverted_hammer.rs`
- **Task**: Update `cdl_inverted_hammer_arrow` macro call (same pattern as hammer).

### 4. `kand/src/ta/ohlcv/cdl_long_shadow.rs`
- **Task**: Update `cdl_long_shadow_arrow` macro call (same pattern as hammer).

### 5. `kand/src/ta/ohlcv/cdl_marubozu.rs`
- **Task**: Update `cdl_marubozu_arrow` macro call (same pattern as hammer).

### 6. `kand/src/ta/stats/correl.rs`
- **Task**: Update `correl_arrow` macro call.
- **Outputs**: `output_correl: TAFloat, output_sum_0: TAFloat, output_sum_1: TAFloat, output_sum_0_sq: TAFloat, output_sum_1_sq: TAFloat, output_sum_01: TAFloat`
- **Return Type**: `TAArrowArray` (x6)

### 7. `kand/src/ta/stats/stddev.rs`
- **Task**: Update `stddev_arrow` macro call.
- **Outputs**: `output_stddev: TAFloat, output_sum: TAFloat, output_sum_sq: TAFloat`
- **Return Type**: `TAArrowArray` (x3)

### 8. `kand/src/ta/stats/var.rs`
- **Task**: Update `var_arrow` macro call.
- **Outputs**: `output_var: TAFloat, output_sum: TAFloat, output_sum_sq: TAFloat`
- **Return Type**: `TAArrowArray` (x3)

### 9. `kand/src/ta/ohlcv/t3.rs`
- **Task**: Update `t3_arrow` macro call.
- **Outputs**: 7 `TAFloat` outputs.
- **Return Type**: 7 `TAArrowArray`s.

### 10. `kand/src/ta/ohlcv/aroon.rs`
- **Task**: Update `aroon_raw` and `aroon` to use `TAInt` for `usize` outputs and replace manual arrow implementation with macro.
- **Outputs**: 4 `TAFloat` + 2 `TAInt`.
- **Return Type**: 4 `TAArrowArray` + 2 `TAArrowIntArray`.

### 11. `kand/src/ta/ohlcv/aroonosc.rs`
- **Task**: Update `aroonosc_raw` and `aroonosc` to use `TAInt` for `usize` outputs and replace manual arrow implementation with macro.
- **Outputs**: 3 `TAFloat` + 2 `TAInt`.
- **Return Type**: 3 `TAArrowArray` + 2 `TAArrowIntArray`.

## Verification Plan
- Run `cargo check -p kand --features arrow` after each file update or in batches.
- Ensure all tests pass with `cargo test -p kand --features arrow`.
