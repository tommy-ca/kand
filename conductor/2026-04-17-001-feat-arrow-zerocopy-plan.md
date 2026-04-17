---
title: "feat: Arrow Zero-Copy Integration"
type: feat
status: active
date: 2026-04-17
---

# Arrow Zero-Copy Integration

## Overview

Integrate Arrow zero-copy data formats directly into the `kand` core library and its language bindings (`kand-py` and `kand-wasm`). This allows efficient, copy-free data exchange with the broader Arrow ecosystem (e.g., Polars, PyArrow, Pandas 2.0, and JS Arrow libraries), significantly improving throughput and reducing memory overhead for large datasets.

## Problem Frame

Currently, `kand-py` and `kand-wasm` allocate and copy memory when bridging data between Rust and host languages (e.g., converting NumPy arrays to Rust `Vec` and back, or passing `Vec<f64>` to WebAssembly). For quantitative finance workflows involving millions of rows, memory copies dominate execution time. By making Arrow a first-class citizen in the core library, we can eliminate this overhead using the Arrow C Data Interface and PyCapsule standard.

## Requirements Trace

- **R1.** Core `kand` exposes an `arrow` feature flag with zero-copy indicator functions taking and returning `arrow::array::PrimitiveArray<T>`.
- **R2.** `kand-py` leverages `pyo3-arrow` to consume and return zero-copy PyCapsules, seamlessly integrating with PyArrow and Polars.
- **R3.** `kand-wasm` avoids copying `Vec<f64>` between JS and WASM contexts, utilizing shared memory views that are resilient to WASM memory growth.

## Scope Boundaries

- **In Scope:** Adding Arrow implementations for indicator functions alongside existing slice-based ones. Updating Python and WASM bindings to use these new implementations. Handling Arrow offsets and null bitmaps.
- **Out of Scope:** Removing or replacing the existing slice-based or NumPy-based APIs.

## Context & Research

### Relevant Code and Patterns

- `kand/src/ta/ohlcv/sma.rs` demonstrates the raw slice implementation (`sma_raw`). Arrow implementations reuse these by extracting slices from `PrimitiveArray`.
- `pyo3-arrow` (v0.11.0) provides the PyCapsule handshake for PyO3 0.25.1.
- `arrow::buffer::MutableBuffer` and `arrow::array::PrimitiveArray` are the primary types for constructing results.

## Key Technical Decisions

- **Core Dependency:** `arrow-rs` (v56.0) added as an optional feature (`arrow`) to `kand`.
- **Null & Offset Handling:** Arrow-native functions will check `array.null_count()`. If nulls are present, they will be handled based on feature flags (e.g., erroring or treating as NaN). Offsets will be respected by using `array.values()[array.offset()..]`.
- **Memory Management (WASM):** Use a `WasmBuffer` wrapper in Rust that manages a pre-allocated, 64-byte aligned buffer. JavaScript will request a new view *after* each call that could cause memory growth to prevent stale views.
- **Precision Agnostic:** Use a `TAArrowArray` type alias (mapped to `Float32Array` or `Float64Array`) to maintain parity with `f32`/`f64` feature flags.
- **Scaling Strategy:** Unit 4 will first normalize all indicators to have public `_raw` slice variants before generating Arrow wrappers via macros.

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The implementing agent should treat it as context, not code to reproduce.*

```rust
// Core (kand) - pseudo-code
#[cfg(feature = "arrow")]
pub fn sma_arrow(input: &TAArrowArray, period: usize) -> Result<TAArrowArray, KandError> {
    // 1. Validation (matches safe slice API)
    lookback(period)?; 
    
    // 2. Handle offsets and nulls
    let input_slice = &input.values()[input.offset()..];
    if input.null_count() > 0 { return Err(KandError::NotImplemented("Null handling")); }

    // 3. Allocate 64-byte aligned mutable buffer
    let mut buffer = MutableBuffer::new(input.len() * size_of::<TAFloat>());
    buffer.resize(input.len() * size_of::<TAFloat>(), 0);
    let output_slice = buffer.typed_data_mut::<TAFloat>();
    
    // 4. Reuse core logic
    sma_raw(input_slice, period, output_slice);
    
    Ok(TAArrowArray::new(buffer.into(), None))
}
```

## Implementation Units

- [ ] **Unit 1: Core Dependency and Foundation**
  **Goal:** Add `arrow` feature and implement `TAArrowArray` type aliases and `sma_arrow` POC.
  **Files:** `Cargo.toml`, `kand/Cargo.toml`, `kand/src/ta/ohlcv/sma.rs`, `kand/src/ta/types.rs`
  **Approach:** Use `MutableBuffer` for aligned allocation. Implement offset handling.

- [ ] **Unit 2: Python Binding (pyo3-arrow v0.11.0)**
  **Goal:** Integrate `pyo3-arrow` into `kand-py` using PyCapsule interface.
  **Files:** `kand-py/Cargo.toml`, `kand-py/src/ta/ohlcv/sma.rs`
  **Approach:** Match version to PyO3 0.25.1.

- [ ] **Unit 3: Safe WASM Buffer Management**
  **Goal:** Implement `WasmBuffer` in `kand-wasm` to provide growth-resilient views for JS.
  **Files:** `kand-wasm/src/ta/ohlcv/sma.rs`, `kand-wasm/src/lib.rs`
  **Approach:** Expose a method to JS that returns a fresh `Float64Array` view on demand. Ensure 64-byte alignment.

- [ ] **Unit 4: Indicator Normalization & Macro Scaling**
  **Goal:** Ensure all 50+ indicators have `_raw` variants, then scale Arrow wrappers using macros.
  **Files:** `kand/src/ta/ohlcv/*.rs`, `kand-py/src/ta/ohlcv/*.rs`, `kand/src/helper/arrow_macro.rs`
  **Approach:** Refactor multi-output indicators (MACD, BBands) to use standardized result structs for both slice and Arrow variants.

## System-Wide Impact

- **Performance:** Zero-copy across all bindings.
- **Safety:** Explicit handling of Arrow offsets/nulls prevents data corruption.

## Risks & Dependencies

- **WASM Memory Growth:** Stale JS views are a primary risk; implementation MUST enforce view refresh.
- **Macro Complexity:** Multi-output indicators require custom result structs.