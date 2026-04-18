# Technical Specification: Arrow Zero-Copy Integration (Updated Phase 6 Strategy)

## Overview
This document specifies the technical architecture for integrating Apache Arrow as a first-class, zero-copy data format in the `kand` ecosystem.

## 1. Architecture

### 1.1 Multi-Tier Indicator Contract
Every indicator follows a consistent implementation hierarchy:
- **`_raw`**: Unvalidated, high-performance computational core.
- **Safe Wrapper**: Parameter and data validation, NaN handling.
- **`_arrow`**: Zero-copy batch processing using Arrow arrays and `BlockPool`.
- **`Stateful` (Indicator trait)**: Encapsulated single-stream state management.
- **`Batch` (BatchIndicator trait)**: Vectorized multi-stream parallel processing.

## 2. Enterprise Durability (V5 Standard)

### 2.1 Columnar Persistence
Internal state scalars are stored as **prefixed constant columns** (`__kand_`) in the state `RecordBatch`. This ensures persistence integrity across standard Arrow processing tools.

### 2.2 Transactional Integrity
Multi-component indicators must implement the **Explicit Commit Pattern**:
1. Clone current state.
2. Attempt component updates on the clone.
3. Commit the clone to `self` only if all sub-operations succeed.

## 3. Phase 6 Scaling Strategy (The Universal Macro)

To scale stateful/batch support from ~4% to 100% of the library, a new **`kand_indicator!`** macro ecosystem is established.

### 3.1 Macro Classification
Indicators are classified into three types for automated generation:
- **Sliding Window**: Requires a buffer of previous $N$ values (e.g., SMA, MOM, ROC).
- **Recursive**: Depends on the previous single output value (e.g., EMA, RSI, ATR).
- **Composite**: Composed of other stateful indicators (e.g., MACD, BBands).

### 3.2 Automation Goals
- Automated generation of `Stateful` structs and `Batch` managers.
- Automated `RecordBatch` serialization following V5 standards.
- Automated `BlockPool` integration for zero-allocation streaming.

## 4. Quality & Verification
- **TDD Parity**: Mandatory numerical parity between all variants.
- **Prek Gate**: Automated verification of formatting, linting (Clippy/Ruff), and type safety (Ty).
- **Performance**: Goal of <15% overhead for Arrow variants vs. raw loops.
