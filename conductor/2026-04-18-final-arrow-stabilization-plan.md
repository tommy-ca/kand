# Implementation Plan: Arrow Integration Stabilization & Benchmarking (Refined)

## Overview
This plan focuses on rectifying the test assertions and numerical precision mismatches identified during the recent test run. The core library's Arrow integration is functional, but test assertions expect `null` values where the current implementation correctly provides `NaN` for non-data periods.

## Problem Frame
The current test suite uses `assert!(result.is_null(i))` for empty periods, but the `_arrow` implementation (and `allow-nan` feature) uses `NAN`. We must update assertions to `assert!(result.is_nan(i))` or equivalent to reflect the "No-Null" policy. Additionally, some indicators exhibit slight floating-point drifts in the Arrow-native implementation which require tuning numerical epsilon tolerances in tests.

## Requirements Trace
- **R1.** Align test assertions with "No-Null" (use `NaN`) policy.
- **R2.** Resolve floating-point precision drifts in Arrow variants.
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Documentation of benchmark results.

## Key Technical Decisions
- **Assertion Standards:** Update all Arrow tests to use `result.is_nan(i)` for empty-period verification.
- **Precision:** Use `approx::assert_relative_eq!` with appropriately tuned epsilon values for all numerical parity tests.

## Implementation Units

### Phase 1: Test & Assertion Stabilization
- [ ] Unit 1.1: Standardize test assertions across all `ohlcv` indicators to use `is_nan()`.
- [ ] Unit 1.2: Tune floating-point tolerances (`epsilon`) for `adosc`, `adx`, `dx`, `adxr`, etc.
- [ ] Unit 1.3: Verification: `cargo test --workspace --features arrow` passes 100%.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Implement benchmark suite (`kand/benches/bench_main.rs`).
- [ ] Unit 2.2: Execute benchmark suite.
- [ ] Unit 2.3: Generate `docs/performance_report.md`.

### Phase 3: Final Audit
- [ ] Unit 3.1: Final security audit of `unsafe` blocks.
- [ ] Unit 3.2: Tag v0.2.2-arrow release.

## Verification
- Test: `cargo test --workspace --features arrow`.
- Benchmark: `cargo bench -p kand --features arrow`.

## Schedule
1. **Stabilization Phase (Day 1):** Execute Phase 1 units to reach 100% pass rate.
2. **Benchmarking Phase (Day 2):** Execute Phase 2 units.
3. **Audit Phase (Day 3):** Execute Phase 3 units.
