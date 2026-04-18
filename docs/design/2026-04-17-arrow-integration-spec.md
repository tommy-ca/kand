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

## 3. Universal Macro Strategy

To scale stateful/batch support library-wide, a "Meta-Compiler" macro system is established in `kand/src/helper/arrow_macro.rs`.

### 3.1 Macro Tiers
- **`kand_arrow_wrapper!`**: Functional Arrow automation.
- **`kand_indicator!`**: Stateful/Batch automation.
- **Goal**: Transition from manual implementation of tiers to unified declarations.

### 4. Enterprise Durability (V5 Standard)

#### 4.1 Transactional Persistence
All state scalars are stored as **prefixed constant columns** (`__kand_`) in the state `RecordBatch`. This ensures persistence integrity across standard Arrow processing tools (Polars/DataFusion).

#### 4.2 Explicit Commit Pattern
Multi-component indicators (e.g., `MACD`) implement a **Transactional Commit** pattern. Sub-components are updated tentatively on state clones; parent state is only modified if all sub-operations succeed.

## 5. Engineering Standards & Quality Assurance
- **TDD Parity**: Mandatory numerical parity between all variants.
- **Prek Gate**: Automated verification of formatting, linting (Clippy/Ruff), and type safety (Ty).
- **Performance**: Goal of <15% overhead for Arrow variants vs. raw loops.
