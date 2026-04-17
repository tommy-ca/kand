# Technical Specification: Arrow Zero-Copy Integration

## Overview
This document specifies the technical architecture for integrating Apache Arrow as a first-class, zero-copy data format in the `kand` ecosystem. It incorporates learnings from initial research and security/feasibility reviews.

## 1. Architecture

### 1.1 Core `kand` Crate
The core crate will be refactored to support Arrow natively via an optional `arrow` feature.

#### 1.1.1 Normalization of Indicators
Every indicator (e.g., `sma`) will expose a `_raw` function with the following signature:
```rust
pub fn sma_raw(input: &[TAFloat], opt_period: TAPeriod, output: &mut [TAFloat]);
```
This ensures that the core computational logic is separated from validation and data-wrapping logic, enabling reuse by Arrow and standard slice wrappers.

#### 1.1.2 Arrow-Native Wrappers
Arrow wrappers will be implemented using the `arrow-rs` crate.
- **Null Handling:** Initial implementation will explicitly reject arrays with null values.
- **Offset Handling:** Functions must correctly calculate the starting pointer by adding `array.offset()` to the underlying buffer's base address.
- **Allocation:** Output buffers must be 64-byte aligned. Use `MutableBuffer` for construction.
- **Type Safety:** Use a `TAArrowArray` type alias that maps to `Float32Array` or `Float64Array` based on precision features.

### 1.2 Python Bindings (`kand-py`)
- **PyCapsule Handshake:** Use `pyo3-arrow` (v0.11.0) to implement the Arrow PyCapsule interface.
- **Zero-Copy Entry Points:** New `_arrow` suffixed functions will be added to the Python module.
- **Macro Generation:** A Rust macro in `kand-py` will automate the generation of these wrappers to minimize boilerplate.

### 1.3 WebAssembly Bindings (`kand-wasm`)
- **WasmBuffer Protocol:** A `WasmBuffer<T>` struct will manage pre-allocated, 64-byte aligned memory.
- **Growth-Resilient Views:** JavaScript must request a fresh `Float64Array` view after any call that potentially grows the WASM linear memory.
- **Protocol Contract:** 
  1. JS requests buffer pointer/length.
  2. JS writes data to view.
  3. Rust processes data in-place or into a pre-allocated output buffer.
  4. JS reads data from view.

## 2. Macro Strategy
To support 50+ indicators, a `kand_arrow_wrapper!` macro will be developed.
- **Standard Inputs:** Handles OHLCV arrays, periods, and smoothing factors.
- **Multi-Output Support:** Supports indicators like MACD and BBands by returning standardized result structs (e.g., `MacdResultArrow`).

## 3. Security & Safety
- **Memory Safety:** Use `MutableBuffer::typed_data_mut` to avoid manual pointer arithmetic where possible.
- **Validation:** All Arrow wrappers must call the same `lookback` and parameter validation logic as the safe slice APIs.
- **Pointer Validation (WASM):** Ensure that pointers exposed to JS are strictly bounded to the pre-allocated `WasmBuffer`.

## 4. Performance Goals
- **Interop:** Zero-copy transfer from `polars.Series` and `pyarrow.Array`.
- **Memory:** Elimination of redundant `Vec<f64>` allocations in bindings.
- **Benchmarks:** Comparison between NumPy-based and Arrow-based throughput.
