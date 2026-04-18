# Implementation Plan: V5 - Enterprise Durability & Quality Hardening

## Overview
This plan implements the V5 "Enterprise Durability" refinements, focusing on columnar state persistence and atomic multi-component updates. We prioritize hardening the core indicators (SMA, EMA, MACD) to establish the project standard.

## Status: IN PROGRESS
V5 Standards defined in `docs/design/2026-04-17-arrow-integration-spec.md`.

## Requirements Trace
- **R1. Columnar Persistence**: Store scalars as `__kand_` prefixed columns in `RecordBatch`.
- **R2. Atomic Commit Pattern**: Ensure multi-component indicators (MACD) are atomic via state cloning.
- **R3. Quality Hardening**: Enforce 100% test coverage for state save/load and error recovery.

## Implementation Units

### Phase 1: Columnar Persistence Refactor
- [x] Unit 1.1: Refactor `BatchSMA` to use prefixed columnar storage.
- [x] Unit 1.2: Refactor `BatchEMA` to use prefixed columnar storage.
- [ ] Unit 1.3: Add a "Cross-Tool Interop" test in `kand/tests/durability_tests.rs` (using simple mock joins).

### Phase 2: Atomic State Management
- [x] Unit 2.1: Implement atomic updates for `StatefulMACD` and `BatchMACD` (using `clone()`).
- [ ] Unit 2.2: Add unit tests in `macd.rs` that verify state remains unchanged if an update fails (e.g. invalid parameter after partial update).

### Phase 3: Documentation & Binding Alignment
- [ ] Unit 3.1: Update Python bindings to support the new `restore_from_record_batch` naming.
- [ ] Unit 3.2: Update WASM bindings for V5 persistence.
- [ ] Unit 3.3: Final audit of the release branch.

## Verification
- `cargo test --workspace --features arrow` (PASS).
- `prek run --all-files` (PASS).
- Numerical and state parity verified.
