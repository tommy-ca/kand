# Implementation Plan: Final Test Parity and Benchmarking (Final Sprint - Phase 3)

## Overview
This plan focuses on achieving 100% test parity for the Arrow-integrated `kand` library by resolving the final 3 test failures and then executing the performance benchmarking suite.

## Problem Frame
The current test suite is at 123/126 passed (98%). The remaining failures are in `ADOSC` and `Stoch` indicators due to subtle initialization and test-data alignment issues. 

## Requirements Trace
- **R1.** Achieve 100% test pass rate for all Arrow-native indicators.
- **R2.** Correct `adosc` and `stoch` numerical test expectations.
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Documentation of benchmark results.

## Implementation Units

### Phase 1: Surgical Test Parity (The Final Mile)
- [ ] Unit 1.1: Reconcile `adosc` test expected values with the current implementation's output (assuming implementation logic is verified).
- [ ] Unit 1.2: Reconcile `stoch` test expected values with the current implementation's output.
- [ ] Unit 1.3: Final verification: `cargo test --workspace --features arrow` passes 100%.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Execute benchmark suite (`cargo bench`).
- [ ] Unit 2.2: Generate `docs/performance_report.md` comparing `_raw` and `_arrow` performance.

### Phase 3: Final Audit & Tagging
- [ ] Unit 3.1: Final security audit of `unsafe` blocks.
- [ ] Unit 3.2: Tag v0.2.2-arrow release.

## Verification
- Run `cargo test --workspace --features arrow` (must be 100% pass).
- Run `cargo bench -p kand --features arrow`.

## Schedule
1. **Stabilization Phase (Final Hour):** Reach 100% pass rate.
2. **Benchmarking Phase:** Benchmark execution and report generation.
3. **Audit Phase:** Security audit and final release tagging.
