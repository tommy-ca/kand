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
    - Generated via `kand_arrow_wrapper!`, `kand_arrow_wrapper_multi!`, or `kand_arrow_wrapper_int!`.
    - Must respect `offset()` and reject `nulls`.

### 1.2 Python & WASM Bindings
#### 1.2.1 Python Bindings (`kand-py`)
- **PyCapsule Handshake:** Use `pyo3-arrow` (v0.11.0) to implement the Arrow PyCapsule interface.
- **Zero-Copy Entry Points:** New `_arrow` suffixed functions are added to the Python module.
- **Macro Scaling Strategy:**
  - **`kand_py_arrow_wrapper!`**: Generates Python bindings for single-output Arrow functions.
  - **`kand_py_arrow_wrapper_multi!`**: Generates Python bindings for multi-output Arrow functions, returning Python tuples of `PyArray`. Supports heterogeneous output types (e.g., mixing float and integer arrays).
  - **`kand_py_arrow_wrapper_int!`**: Generates Python bindings for single-output Arrow functions returning integer arrays (pattern recognition).
  - Supports output counts from 2 to 7 to cover all complex indicators (MACD, BBands, ADX, etc.).
  - Releases the GIL during computation using `py.allow_threads`.

#### 1.2.2 WebAssembly Bindings (`kand-wasm`)
- **WasmBuffer Protocol:** A `WasmBuffer` struct manages pre-allocated, memory-growth-resilient memory. It supports generic types via `as_slice::<T>` and `as_mut_slice::<T>`.
- **Protocol Contract:** JS requests pointer, writes data, calls Rust, and reads results from the shared view.
- **Incremental Multi-Output:** Uses `Result` structs (e.g., `SupertrendResult`) for returning multiple values efficiently.

## 2. Test-Driven Development (TDD) Standard
Every Arrow-native implementation must be preceded or accompanied by a test case that:
1.  **Validation Parity:** Proves that `_arrow` variants reject the same invalid parameters as the safe slice variants.
2.  **Numerical Parity:** Proves that `_arrow` variants produce bit-identical results to safe slice variants (modulo `allow-nan` behavior).
3.  **Offset Integrity:** Proves that the implementation correctly respects Arrow array offsets by testing with sliced input arrays.
4.  **Alignment Check:** Proves that output buffers are 64-byte aligned (verified via `MutableBuffer` address).

## 3. Macro Strategy
To support 50+ indicators efficiently, three primary macros are utilized:
- **`kand_arrow_wrapper!`**: Automates Arrow variants for single-output floating-point indicators.
- **`kand_arrow_wrapper_multi!`**: Automates Arrow variants for multi-output indicators (e.g., MACD, BBands, CDL patterns).
  - Handles the allocation of multiple `MutableBuffer` instances.
  - Returns a tuple of Arrow arrays, explicitly typed (e.g., `TAArrowArray` and `TAArrowIntArray`).
  - Ensures numerical and validation parity across all outputs.
- **`kand_arrow_wrapper_int!`**: Automates Arrow variants for single-output integer indicators (e.g., candle patterns).
  - Uses `TAArrowIntArray`.

**Macro Parameters:**
- `inputs`: Defines required input arrays.
- `params`: Defines indicator-specific parameters (e.g., `opt_period: usize`).
- `lookback_params`: Maps only the parameters needed by the specific `lookback()` function.
- `outputs` / `return_type`: Exclusively in `multi` macros to define precise buffer typing for each output.
