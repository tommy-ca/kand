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

### Phase 1: Normalization & Foundation (COMPLETED)
- [x] Unit 1.1: Foundation established (`arrow` feature, type aliases).
- [x] Unit 1.2: SMA POC across core, python, wasm.
- [x] Unit 1.3: Multi-output macro defined.

### Phase 2: Batch-Driven TDD Scaling
Each batch follows a **Specs -> Tests -> Impl -> Audit** cycle.

- [ ] **Unit 2.1: Batch 2 - BBands & ADX (Multi-Output Focus)**
  **Goal:** Refactor BBands and ADX to use multi-output macros and normalize core logic.
  **Files:** `kand/src/ta/ohlcv/bbands.rs`, `kand/src/ta/ohlcv/adx.rs`
  **Approach (TDD):**
  1. Define spec for Arrow result structs.
  2. Write Arrow parity tests (offset, alignment).
  3. Implement `_raw` and `_arrow` variants.
  4. Perform semantic commit.

- [ ] **Unit 2.2: Batch 3 - Momentum & Volatility (RSI, ATR, etc.)**
  **Goal:** Systematic normalization of remaining OHLCV indicators.
  **Files:** `kand/src/ta/ohlcv/rsi.rs` (Done), `kand/src/ta/ohlcv/atr.rs` (Done), others.

- [ ] **Unit 2.3: Batch 4 - Stats (StdDev, Var, etc.)**
  **Goal:** Normalize and wrap statistics functions.
  **Files:** `kand/src/ta/stats/*.rs`

### Phase 3: Bindings & Audit

- [ ] **Unit 3.1: Python Binding Auto-Scaling**
  **Goal:** Develop `kand_py_arrow_wrapper!` macro to scale `kand-py` without manual boilerplate.
  **Files:** `kand-py/src/ta/ohlcv/*.rs`

- [ ] **Unit 3.2: Performance & Security Audit**
  **Goal:** Final pass with specialized agents to verify zero-copy and memory safety.
  **Files:** `docs/audits/*.md`

## Risks & Dependencies
- **WASM Memory Growth:** Critical that JS refreshes views.
- **Alignment Errors:** Must strictly use `MutableBuffer` or `arrow::buffer::alloc`.
- **Dependency Conflicts:** Ensuring `pyo3-arrow` matches our PyO3 version.
