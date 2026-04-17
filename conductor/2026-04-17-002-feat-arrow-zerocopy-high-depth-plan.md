# Plan: Arrow Zero-Copy Integration (High-Depth)

## Overview
This plan implements the "Arrow First" vision for the `kand` project. It introduces native Apache Arrow support into the Rust core and optimizes the Python and WASM bindings for zero-copy data transfer. 

## Problem Frame
The current `kand` bindings suffer from memory copy overhead. By integrating `arrow-rs` and `pyo3-arrow`, we enable zero-copy interoperability with Polars, PyArrow, and modern JS data libraries, halving memory usage and significantly improving throughput for large datasets.

## Requirements Trace
- **R1.** Core Arrow Support (`arrow` feature in `kand`).
- **R2.** Zero-Copy Indicators (Arrow-native variants).
- **R3.** Offset & Null Safety (Strict erroring on nulls).
- **R4.** Python Zero-Copy (`pyo3-arrow` / PyCapsule).
- **R5.** WASM Zero-Copy (Growth-resilient shared buffers).
- **R6.** Precision Parity (`f32`/`f64` support).
- **R7.** API Parity (Scaling to all indicators).

## Key Technical Decisions
- **Alignment:** Output buffers will be 64-byte aligned via `MutableBuffer`.
- **Safety:** Explicit `lookback` validation and null checks in all Arrow variants.
- **WASM Protocol:** Implementing a `refreshView()` requirement for JS-side buffer access.
- **Normalization:** Refactoring all indicators to expose a `_raw` slice-based function.

## Implementation Units

### Phase 1: Normalization & Foundation

- [ ] **Unit 1.1: Project-Wide Normalization**
  **Goal:** Ensure every indicator in `kand/src/ta/ohlcv/` and `kand/src/ta/stats/` has a public `_raw` variant.
  **Files:** `kand/src/ta/**/*.rs`
  **Approach:** Extract core logic from current safe wrappers into `_raw` functions taking slices. This is a prerequisite for macro-based Arrow wrapping.
  **Test scenarios:** Verify existing tests still pass and use the new `_raw` internally.

- [ ] **Unit 1.2: Core Arrow Foundation**
  **Goal:** Add `arrow` feature, dependency, and type aliases.
  **Files:** `kand/Cargo.toml`, `Cargo.toml`, `kand/src/ta/types.rs`
  **Approach:** Add `arrow = { version = "56.0", optional = true }`. Define `TAArrowArray` and `TAArrowBuilder` aliases that switch based on `f32`/`f64` features.

### Phase 2: Implementation & Bindings

- [ ] **Unit 2.1: SMA Arrow Implementation (POC)**
  **Goal:** Implement `sma_arrow` in the core library and test for correctness and alignment.
  **Files:** `kand/src/ta/ohlcv/sma.rs`
  **Approach:** Follow the pattern: Validate -> Handle Offsets -> Allocate Aligned -> Call `sma_raw` -> Return `PrimitiveArray`.
  **Verification:** Use `MutableBuffer::is_aligned` or check address modulo 64.

- [ ] **Unit 2.2: Python PyCapsule Binding**
  **Goal:** Integrate `pyo3-arrow` (v0.11.0) and expose `sma_arrow_py`.
  **Files:** `kand-py/Cargo.toml`, `kand-py/src/ta/ohlcv/sma.rs`
  **Approach:** Implement the `PyArray` handshake. Ensure Polars/PyArrow can consume the output without copying.

- [ ] **Unit 2.3: WASM Shared Buffer Manager**
  **Goal:** Implement a growth-resilient `WasmBuffer` and expose `sma_wasm_zero_copy`.
  **Files:** `kand-wasm/src/ta/ohlcv/sma.rs`, `kand-wasm/src/lib.rs`
  **Approach:** Managed `Vec<T>` with exposed pointer. JS side logic for view refreshing.

### Phase 3: Scaling & Audit

- [ ] **Unit 3.1: Macro Scaling (Core & Python)**
  **Goal:** Develop and apply macros to generate Arrow variants for all normalized indicators.
  **Files:** `kand/src/helper/arrow_macro.rs`, `kand-py/src/ta/ohlcv/*.rs`
  **Approach:** Standardize result structs for multi-output indicators (MACD, BBands).

- [ ] **Unit 3.2: Zero-Copy Performance Audit**
  **Goal:** Run benchmarks and memory profiling to verify zero-copy goals.
  **Files:** `kand-py/python/benches/bench_arrow.py`
  **Approach:** Compare throughput of `sma_py` (Numpy-based) vs `sma_arrow_py` (Arrow-based) for 10M rows.

## Risks & Dependencies
- **WASM Memory Growth:** Critical that JS refreshes views.
- **Alignment Errors:** Must strictly use `MutableBuffer` or `arrow::buffer::alloc`.
- **Dependency Conflicts:** Ensuring `pyo3-arrow` matches our PyO3 version.
