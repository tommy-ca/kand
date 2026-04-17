---
date: 2026-04-17
topic: arrow-integration
---

# Arrow Zero-Copy Integration: Requirements & Design

## Problem Frame
The current `kand` library suffers from significant memory copy overhead in its Python (`kand-py`) and WebAssembly (`kand-wasm`) bindings. Quantitative finance workflows often involve millions of data points; copying these between the Rust core and the host environment (NumPy, JS, etc.) creates a performance bottleneck and doubles memory consumption. 

The goal is to modernize `kand` by making Apache Arrow a first-class, zero-copy data format, enabling seamless integration with the modern data ecosystem (Polars, PyArrow, Pandas 2.0).

## Requirements
- **R1. Core Arrow Support:** The `kand` core crate must expose an optional `arrow` feature.
- **R2. Zero-Copy Indicators:** All technical indicators must provide Arrow-native variants (e.g., `sma_arrow`) that accept and return `arrow::array::PrimitiveArray<T>` without copying the underlying buffer.
- **R3. Offset & Null Safety:** Arrow-native functions must correctly handle Arrow array offsets and explicitly reject (error on) arrays with null values in the initial implementation.
- **R4. Python Zero-Copy:** `kand-py` must use `pyo3-arrow` and automated macros (`kand_py_arrow_wrapper!`) to exchange data via the Arrow PyCapsule interface.
- **R5. WASM Zero-Copy:** `kand-wasm` provides `WasmBuffer` for growth-resilient shared buffers, enabling zero-copy between JS and Rust.
- **R6. Precision Parity:** The Arrow implementation respects existing `f32`/`f64` precision features via `TAArrowArray` aliases.
- **R7. Systematic Normalization:** Every indicator follows the `_raw` -> `Safe Wrapper` -> `_arrow` three-tier contract.

## Current Progress
- **Phase 1 Foundation:** Complete (Macros, Foundation, SMA POC).
- **Phase 2 Scaling:** 
  - Batch 1 & 2 (Core & Multi-Output): Complete.
  - Batch 3 (Trend Extensions): Complete.
  - Batch 4 (Momentum & Volume): In Progress.

## Success Criteria
- Benchmarks show near-zero overhead for data transfer between Python/JS and Rust.
- Memory usage for large batch calculations is reduced by ~50% (no duplicate buffers).
- Full integration tests confirm identical results between slice-based and Arrow-based implementations.

## Scope Boundaries
- **In Scope:** `arrow-rs` integration, `pyo3-arrow` bindings, WASM shared buffer manager, macro-based wrapper generation.
- **Out of Scope:** Multi-dimensional Arrow arrays (tensors), Arrow RecordBatches (initially limited to Arrays), automatic null-to-nan conversion (strict erroring only).

## Key Decisions
- **Strict Null Policy:** For the MVP, `kand` will return `KandError::InvalidData` if an input Arrow array has a `null_count > 0`. This avoids the complexity of null-to-nan conversion while ensuring correctness.
- **MutableBuffer Allocation:** Output arrays will be pre-allocated using `MutableBuffer` with 64-byte alignment to comply with Arrow standards.
- **Macro Scaling:** A central macro will be used to generate Arrow and Python wrappers for all indicators to ensure consistency and maintainability.
- **WASM Buffer Protocol:** JS will be required to refresh its `Float64Array` view after every WASM call that could trigger a memory grow operation.

## High-Level Technical Direction

### Core (Rust)
Indicators will be refactored to always expose a `_raw` variant taking `&[T]`. The `_arrow` variants will:
1. Validate inputs (lookback, etc.).
2. Slice the input buffer based on the Arrow `offset()`.
3. Allocate an aligned `MutableBuffer`.
4. Call the `_raw` function on the slices.
5. Return a `PrimitiveArray<T>`.

### Python (PyO3)
Use `pyo3-arrow` to extract `PrimitiveArray` from `PyArray` (PyCapsule). The wrapper will be generated via a macro in `kand-py`.

### WASM (wasm-bindgen)
Implement a `WasmBuffer` struct that holds a `Vec<T>` (pre-allocated and leaked/managed). Expose the pointer and length to JS. JS uses `new Float64Array(wasm.memory.buffer, ptr, len)`.

## Outstanding Questions

### Deferred to Planning
- [Technical] How to handle indicators that return multiple arrays (MACD, BBands) in a standardized way?
- [Needs research] Performance comparison between `MutableBuffer` and `Vec::with_capacity` followed by `Buffer::from_vec`.

## Next Steps
→ `/ce:plan` for structured implementation planning.
