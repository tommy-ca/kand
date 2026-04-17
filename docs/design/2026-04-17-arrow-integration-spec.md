# Technical Specification: Arrow Zero-Copy Integration

## Overview
This document specifies the technical architecture for integrating Apache Arrow as a first-class, zero-copy data format in the `kand` ecosystem.

## 1. Architecture

### 1.1 Core `kand` Crate: Normalization Contract
Every indicator must expose a three-tier implementation:
1.  **`_raw` (Computational Core):** High-performance, slice-based, no validation.
    - Signature: `pub fn name_raw(inputs: &[T], params: P, outputs: &mut [T]);`
2.  **Safe Wrapper (Standard API):** Validation, NaN handling, slice-based.
    - Calls `_raw` internally.
3.  **`_arrow` (Modern API):** Zero-copy, Arrow-native, validation-parity.
    - Generated via `kand_arrow_wrapper!` or `kand_arrow_wrapper_multi!`.
    - Must respect `offset()` and reject `nulls`.

### 1.2 Python & WASM Bindings
[Unchanged from previous version]

## 2. Test-Driven Development (TDD) Standard
Every Arrow-native implementation must be preceded or accompanied by a test case that:
1.  **Validation Parity:** Proves that `_arrow` variants reject the same invalid parameters as the safe slice variants.
2.  **Numerical Parity:** Proves that `_arrow` variants produce bit-identical results to safe slice variants (modulo `allow-nan` behavior).
3.  **Offset Integrity:** Proves that the implementation correctly respects Arrow array offsets by testing with sliced input arrays.
4.  **Alignment Check:** Proves that output buffers are 64-byte aligned (verified via `MutableBuffer` address).

## 3. Macro Strategy
To support 50+ indicators efficiently, two primary macros are utilized:
- **`kand_arrow_wrapper!`**: Automates Arrow variants for single-output indicators.
- **`kand_arrow_wrapper_multi!`**: Automates Arrow variants for multi-output indicators (e.g., MACD, BBands).
  - Handles the allocation of multiple `MutableBuffer` instances.
  - Returns a tuple of `TAArrowArray`.
  - Ensures numerical and validation parity across all outputs.

Multi-output indicators should standardize their `_raw` function to take mutable slices for each output in a predictable order.
