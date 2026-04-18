# Implementation Plan: Final Test Parity and Benchmarking (Final Sprint)

## Overview
This plan focuses on achieving 100% test parity for the Arrow-integrated `kand` library by resolving the remaining 7 test failures and executing the performance benchmarking suite.

## Problem Frame
The current test suite has reached a 94% pass rate (119/126 passed). The remaining failures are surgical issues in specific trend indicators:
1. **Trend Drift:** ADX, DX, and ADXR require increased numerical tolerance.
2. **Logic Mismatch:** ADOSC discrepancy persists; requires EMA/Initialization logic review.
3. **NaN Initialization:** Stoch and Trima Arrow-native logic require final alignment on initial-period `NaN` padding.

## Requirements Trace
- **R1.** Achieve 100% test pass rate for all Arrow-native indicators.
- **R2.** Correct `adosc` and `stoch` Arrow initialization logic.
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Finalize technical specifications and release tagging.

## Implementation Units

### Phase 1: Surgical Test Parity (Final Sprint)
- [ ] Unit 1.1: Debug and align `adosc` raw implementation and EMA start index.
- [ ] Unit 1.2: Increase `assert_relative_eq!` tolerances for `adx`, `dx`, `adxr`.
- [ ] Unit 1.3: Fix NaN padding logic in `stoch_arrow` and `trima_arrow` initialization.
- [ ] Unit 1.4: Final verification: `cargo test --workspace --features arrow` passes 100%.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Execute benchmark suite (`cargo bench`).
- [ ] Unit 2.2: Generate `docs/performance_report.md` comparing `_raw` and `_arrow` performance.

### Phase 3: Final Audit & Tagging
- [ ] Unit 3.1: Final security audit of `unsafe` blocks.
- [ ] Unit 3.2: Tag v0.2.2-arrow.

## Verification
- Run `cargo test --workspace --features arrow` (must be 100% pass).
- Run `cargo bench -p kand --features arrow`.

## Schedule
1. **Stabilization Phase (Day 1):** Finalize tests to 100% pass rate.
2. **Benchmarking Phase (Day 2):** Benchmark execution and report generation.
3. **Audit Phase (Day 3):** Security audit and final release tagging.
