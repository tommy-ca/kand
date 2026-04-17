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
- **R7.** Systematic Normalization (`_raw` contract).

## Key Technical Decisions
- **Alignment:** Output buffers are 64-byte aligned via `MutableBuffer`.
- **Safety:** Strict null checks and offset-aware slicing in all Arrow variants.
- **WASM Protocol:** `WasmBuffer` managed memory for zero-copy JS interaction.
- **Scaling:** High-level macros in both `kand` and `kand-py` to minimize boilerplate.

## Implementation Units

### Phase 1: Normalization & Foundation (COMPLETED)
- [x] Unit 1.1: Foundation established (`arrow` feature, type aliases).
- [x] Unit 1.2: SMA POC across core, python, wasm.
- [x] Unit 1.3: Multi-output macros defined.

### Phase 2: Batch-Driven TDD Scaling
Each batch follows a **Specs -> Tests -> Impl -> Audit** cycle.

- [x] **Unit 2.1: Batch 1 & 2 - Core & Multi-Output (COMPLETED)**
  - Done: SMA, EMA, MACD, RSI, ATR, BBands, ADX, DX, Plus DI, Minus DI, Var.

- [x] **Unit 2.2: Batch 3 - Trend & Volatility Extensions (COMPLETED)**
  - Done: DEMA, TEMA, T3, TRIMA, WMA.

- [ ] **Unit 2.3: Batch 4 - Momentum & Volume (IN PROGRESS)**
  - Done: CCI, MFI, TypPrice, OBV, WillR.
  - Pending: AD, AdOsc, ADR, ADXR, Aroon, AroonOsc, BOP.
  **Approach (TDD):**
  1. Refactor core to `_raw` variants.
  2. Implement `_arrow` variants via macro.
  3. Update Python bindings via `kand-py` macros.
  4. Perform semantic commit.

- [ ] **Unit 2.4: Batch 5 - Stats & Others**
  **Goal:** Normalize and wrap remaining stats (StdDev, Sum, Max, Min, Correl).

- [ ] **Unit 2.5: Batch 6 - Candle Patterns**
  **Goal:** Bulk normalization of CDL_* pattern indicators.

### Phase 3: Bindings & Final Audit

- [x] **Unit 3.1: Python Binding Auto-Scaling (COMPLETED)**
  - Macro `kand_py_arrow_wrapper!` and `kand_py_arrow_wrapper_multi!` implemented.

- [ ] **Unit 3.2: Performance & Security Audit**
  **Goal:** Final pass with specialized agents to verify zero-copy and memory safety.
