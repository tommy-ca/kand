# Technical Specification: Arrow Zero-Copy Integration (Updated WASM-V4)

## Overview
This document specifies the technical architecture for integrating Apache Arrow as a first-class, zero-copy data format in the `kand` ecosystem, leveraging `arrow-rs` v58.1.0 and `pyo3-arrow` v0.17.0.

## 1. Architecture

### 1.1 Core `kand` Crate: Normalization Contract
Every indicator exposes a three-tier implementation:
1.  **`_raw`**: High-performance, slice-based, zero validation.
2.  **Safe Wrapper**: Validation, NaN handling, slice-based.
3.  **`_arrow`**: Zero-copy, Arrow-native API with pooling.

### 1.2 Performance Optimization (V3 Core, V4 WASM)
The library utilizes a **Thread-Local Block Cache** to minimize heap allocations.
- **`BlockPool`**: Manages 64-byte aligned memory regions.
- **`PooledAllocation`**: Implements the Arrow `Allocation` trait to return memory to the pool upon buffer drop.
- **WASM Integration (V4)**: `WasmBuffer` is integrated with the core `BlockPool`, providing 64-byte alignment (standard for Arrow/SIMD) and pooled reuse within the WASM linear memory.

### 1.3 Python & WASM Bindings
#### 1.3.1 Python Bindings (`kand-py`)
- **PyO3 0.28 Migration**: Replaced `allow_threads` with `detach` to align with the latest safe GIL management patterns.
- **PyCapsule Handshake**: Fully compliant with the latest Arrow C Data Interface via `pyo3-arrow` v0.17.0.

#### 1.3.2 WebAssembly Bindings (`kand-wasm`)
- **Generic WasmBuffer**: Supports heterogeneous data types (`f64`, `i32`, `i64`) through a unified shared-memory protocol.
- **Arrow Compatibility**: Exposes `_arrow` variants (e.g., `sma_arrow_wasm`) that produce pooled Arrow-native buffers, enabling zero-copy integration with JavaScript Arrow libraries (via the C Data Interface).

## 2. Test-Driven Development (TDD) Standard
1.  **Validation Parity**: Errors match legacy slice variants.
2.  **Numerical Parity**: Bit-identical results (modulo accumulation drift).
3.  **Offset Integrity**: Correct handling of `array.offset()`.
4.  **Alignment**: Guaranteed 64-byte alignment for SIMD compatibility.
5.  **NaN Padding**: Consistent initial-period padding.

## 3. Macro Strategy
To support 75+ indicators efficiently, specialized macros automate Arrow variant generation:
- **`kand_arrow_wrapper!`**: Single-output floating point indicators.
- **`kand_arrow_wrapper_multi!`**: Multi-output indicators (e.g., MACD, BBands). Supports mixed types (e.g., `TAFloat` and `TAInt`).
- **`kand_arrow_wrapper_int!`**: Single-output integer indicators (e.g., candle patterns).

All macros are integrated with the `BlockPool` for automatic memory management across Rust, Python, and WASM.

## 4. Stateful Indicator Framework (Streaming V1)

To support high-density streaming (thousands of assets) and standardized persistence, `kand` provides encapsulated stateful indicator traits.

### 4.1 `Indicator` and `BatchIndicator` Traits
- **`Indicator`**: Standard interface for single-stream stateful indicators. Supports incremental updates via `next()` and state persistence via `to_record_batch()`.
- **`BatchIndicator`**: Optimized for vectorized multi-stream updates. Processes $N$ streams simultaneously using Arrow arrays, leveraging data parallelism and the internal `BlockPool`.

### 4.2 Vectorized State Management
Batch indicators like `BatchSMA` manage internal state using Arrow-native structures:
- **Circular Window Buffer**: Stores historical data for $N$ streams in a single aligned `MutableBuffer`.
- **Zero-Allocation Updates**: High-frequency streaming updates utilize the `BlockPool` to avoid heap thrashing.
- **Interoperability**: Batch states can be exported as `RecordBatch`, enabling state persistence across system restarts or distributed handovers.

### 4.3 Python & WASM Stateful Objects
- **Python**: Batch indicators are exposed as `#[pyclass]` objects (e.g., `kand.BatchSMA`), allowing Python quantitative engines to update thousands of asset states with a single vectorized call.
- **WASM**: Stateful wrappers provide growth-resilient state management within the WASM heap.
