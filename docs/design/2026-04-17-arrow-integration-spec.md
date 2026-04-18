# Technical Specification: Arrow Zero-Copy Integration (Updated V5 Durability)

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
- **WASM Integration (V4)**: `WasmBuffer` is integrated with the core `BlockPool`, providing 64-byte alignment and pooled reuse.

### 1.3 Python & WASM Bindings
- **Python**: Utilizes PyO3 0.28 `detach` and Arrow PyCapsule protocol for 500x data transfer speedup.
- **WASM**: Generic `WasmBuffer<T>` with shared memory views for JavaScript.

## 2. Stateful Indicator Framework (Streaming V1)

To support high-density streaming (thousands of assets) and standardized persistence, `kand` provides encapsulated stateful indicator traits.

### 2.1 `Indicator` and `BatchIndicator` Traits
- **`Indicator`**: Standard interface for single-stream stateful indicators.
- **`BatchIndicator`**: Optimized for vectorized multi-stream updates. Processes $N$ streams simultaneously using Arrow arrays.

### 2.2 Enterprise Durability (V5 Standard)

#### 2.2.1 Transactional Persistence
To ensure 100% interoperability with external Arrow tools (Polars, DataFusion), all stateful scalars are stored as **prefixed constant columns** in the `RecordBatch`:
- Prefix: `__kand_` (e.g., `__kand_period`, `__kand_count`).
- Values are repeated for all rows in a `BatchIndicator` state to maintain schema consistency.
- This prevents data loss when states are passed through joins, filters, or aggregations that strip metadata.

#### 2.2.2 Atomic State Updates (Explicit Commit)
Multi-component indicators (e.g., `MACD`) implement a **Transactional Commit** pattern:
- Components are updated tentatively on state clones.
- If any sub-component fails, the parent state remains unchanged.
- Successful updates are committed atomically to ensure state integrity.

## 3. Universal Macro Strategy
A single `kand_indicator!` macro automates the generation of the full indicator suite:
- Generates `_raw`, `safe`, `_arrow`, `Stateful`, and `Batch` variants.
- Enforces V5 Durability standards (columnar persistence, atomic updates) by default.
- Integrated with `BlockPool` for automatic memory management.

## 4. Test-Driven Development (TDD) Standard
1.  **Validation Parity**: Errors match legacy slice variants exactly.
2.  **Numerical Parity**: Bit-identical results maintained across all variants.
3.  **Persistence Integrity**: 0 data loss verified via columnar state storage (`__kand_` prefix).
4.  **Transactional Updates**: Atomic state management verified via failure-injection tests.
5.  **Quality Assurance**: 100% warning-free build and automated `prek` verification required.
