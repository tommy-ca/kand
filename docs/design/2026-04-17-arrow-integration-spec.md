# Technical Specification: Arrow Zero-Copy Integration (Updated v58.1.0)

## Overview
This document specifies the technical architecture for integrating Apache Arrow as a first-class, zero-copy data format in the `kand` ecosystem, leveraging `arrow-rs` v58.1.0 and `pyo3-arrow` v0.17.0.

## 1. Architecture

### 1.1 Core `kand` Crate: Normalization Contract
Every indicator exposes a three-tier implementation:
1.  **`_raw`**: High-performance, slice-based, no validation.
2.  **Safe Wrapper**: Validation, NaN handling, slice-based.
3.  **`_arrow`**: Zero-copy, Arrow-native API with pooling.

### 1.2 Performance Optimization (V3)
The library utilizes a **Thread-Local Block Cache** to minimize heap allocations.
- **`BlockPool`**: Manages 64-byte aligned memory regions.
- **`PooledAllocation`**: Implements the Arrow `Allocation` trait to return memory to the pool upon buffer drop.
- **Bulk Initialization**: Uses `slice.fill(TAFloat::NAN)` which is optimized by the compiler to SIMD memory-fill instructions.
- **Overhead**: Reduced to **11% - 17%** for batch operations, meeting the high-frequency trading performance requirements.

### 1.3 Python & WASM Bindings
#### 1.3.1 Python Bindings (`kand-py`)
- **PyO3 0.28 Migration**: Replaced `allow_threads` with `detach` to align with the latest safe GIL management patterns.
- **PyCapsule Handshake**: Fully compliant with the latest Arrow C Data Interface via `pyo3-arrow` v0.17.0.

#### 1.3.2 WebAssembly Bindings (`kand-wasm`)
- **Generic WasmBuffer**: Supports heterogeneous data types (`f64`, `i32`, `i64`) through a unified shared-memory protocol.

## 2. Test-Driven Development (TDD) Standard
1.  **Validation Parity**: Errors match legacy slice variants.
2.  **Numerical Parity**: Bit-identical results (modulo accumulation drift).
3.  **Offset Integrity**: Correct handling of `array.offset()`.
4.  **Alignment**: Guaranteed 64-byte alignment for SIMD compatibility.
5.  **NaN Padding**: Consistent initial-period padding.

## 3. Macro Strategy
- **`kand_arrow_wrapper!`**: Single-output float.
- **`kand_arrow_wrapper_multi!`**: Multi-output (supports mixed types).
- **`kand_arrow_wrapper_int!`**: Single-output integer (pattern signals).

All macros are now integrated with the `buffer_pool` for automatic memory management.
