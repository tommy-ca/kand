# Requirements: Multi-Input/Multi-Output Universal Macro (`kand_indicator_multi!`)

## 1. Problem Statement
The current `kand_indicator!` Universal Macro Engine (V6.3) successfully handles single-output scalar logic and single-input circular buffering. However, it fails when encountering complex indicators like `CCI`, `MFI`, and `DX`. 

Specifically, the limitations are:
- **Multi-Input Sliding Windows**: When an indicator requires tracking multiple inputs within a rolling window (e.g., `high`, `low`, `close` for `CCI`), the current macro attempts to store these as flattened tuples which complicates Arrow `FixedSizeListArray` serialization and memory alignment.
- **Multi-Output State**: Indicators that produce multiple parallel arrays (e.g., `DX` yields `DX`, `+DI`, `-DI`, or MACD yielding `MACD`, `Signal`, `Histogram`) cannot be automatically wrapped by a macro that only expects a single scalar `Result<TAFloat, KandError>` from the `next` closure.
- **Extended Buffers**: Some indicators require `period + 1` window sizing (e.g., tracking the previous close alongside a full period of typical prices).

## 2. Requirements

### 2.1 Structs & Traits Generation
A new or extended macro (`kand_indicator_multi!` or an enhancement to `kand_indicator!`) MUST:
1. Generate `StatefulX` and `BatchX` structs.
2. Implement the standard `Indicator` and `BatchIndicator` traits.
3. Accept a variable number of outputs `(O1, O2, ...)` from the `next` and `next_batch` methods.

### 2.2 Storage & Serialization
1. **Parallel Circular Buffers**: Instead of storing tuples `(A, B, C)` in a single circular buffer, the macro MUST generate parallel circular buffers for each input.
   - Example: `__kand_window_high`, `__kand_window_low`, `__kand_window_close`.
2. **Columnar Arrow State**: Each parallel circular buffer MUST be properly serialized as its own `FixedSizeListArray` column with the `__kand_` prefix to adhere to the V5.2 Durability Standard.

### 2.3 Syntax Design
The macro syntax should clearly delineate multiple outputs:
```rust
crate::kand_indicator_multi!(
    CCI,
    type: sliding_window,
    inputs: { high: TAFloat, low: TAFloat, close: TAFloat },
    params: { period: usize },
    state: { sum: TAFloat },
    outputs: { cci: TAFloat }, // Define outputs clearly
    init: |period| { ... },
    next: |state, (high, low, close)| -> Result<(TAFloat), KandError> { ... }
);
```

## 3. Scope of Resolution
This requirement strictly blocks the mass porting of:
- `CCI` (Commodity Channel Index)
- `MFI` (Money Flow Index)
- `DX` (Directional Movement Index)

It is also highly relevant for future ports of composite indicators like `MACD`, `BBANDS`, and `STOCH`.

## 4. Subagent & Skill Utilization
- We will leverage the `codebase_investigator` subagent to audit `kand/src/helper/arrow_macro.rs` and identify insertion points for the new parallel buffer logic.
- We will use `ce:plan` or `document-review` to validate this spec before writing code.
