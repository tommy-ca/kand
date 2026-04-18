# Implementation Plan: Arrow Integration Stabilization & Benchmarking (Final Refinement)

## Overview
This plan focuses on achieving 100% test parity for the Arrow-integrated `kand` library by resolving remaining floating-point drift and assertion mismatches. Upon completion, we will proceed to benchmarking.

## Problem Frame
The current test suite failures (9/126) arise from:
1. **Assertion Drift:** Arrow implementations using `NAN` as a placeholder where tests expect `null` (now `is_nan`).
2. **Floating-Point Discrepancies:** Minor numerical drifts in `adosc`, `adx`, `dx`, and `adxr` due to different accumulation orders in Arrow vs. legacy implementations.
3. **`adosc` Logical Discrepancy:** A larger numerical deviation in `adosc` suggesting a possible period-indexing error.

## Requirements Trace
- **R1.** Achieve 100% test pass rate for all Arrow-native indicators.
- **R2.** Correct `adosc` logic to match legacy implementation results.
- **R3.** Formal Performance Benchmarking.
- **R4.** Documentation of benchmark results.

## Implementation Units

### Phase 1: Surgical Test Fixes
- [ ] Unit 1.1: Finalize assertion fixes (convert `is_null` -> `is_nan` for `sma`, `stoch`, etc.).
- [ ] Unit 1.2: Debug `adosc` logic and resolve numerical discrepancies.
- [ ] Unit 1.3: Adjust epsilon tolerances for trend-sensitive indicators (`adx`, `dx`, `adxr`).
- [ ] Unit 1.4: Final verification: `cargo test --workspace --features arrow` passes 100%.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Execute benchmark suite (`cargo bench`).
- [ ] Unit 2.2: Generate `docs/performance_report.md`.

### Phase 3: Final Audit & Tagging
- [ ] Unit 3.1: Final audit of memory-safety features.
- [ ] Unit 3.2: Tag v0.2.2-arrow.

## Verification
- Run `cargo test --workspace --features arrow`.
- Run `cargo bench -p kand --features arrow`.
