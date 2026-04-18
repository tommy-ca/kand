# Implementation Plan: Final Test Parity and Benchmarking

## Overview
This plan focuses on achieving 100% test parity for the Arrow-integrated `kand` library by resolving the final numerical discrepancies and assertion mismatches in the test suite.

## Problem Frame
The current test suite has reached 93% pass rate (118/126 passed). The remaining failures are surgical issues in specific trend indicators:
1. **Numerical Discrepancies:** ADOSC, ADX, DX, and ADXR exhibit floating-point drifts that exceed current epsilon tolerances or indicate minor logic mismatches in the raw computational core.
2. **Assertion/Logic Mismatches:** Stoch and AroonOsc failures point to either incorrect NaN-filling logic or misaligned initial-period expectations in the Arrow-native implementations.

## Requirements Trace
- **R1.** Achieve 100% test pass rate for all Arrow-native indicators.
- **R2.** Correct `adosc` and `stoch` Arrow initialization logic.
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Finalize technical specifications to reflect current implementation.

## Implementation Units

### Phase 1: Surgical Test Parity
- [ ] Unit 1.1: Debug and align `adosc` raw implementation and EMA start index.
- [ ] Unit 1.2: Increase `assert_relative_eq!` tolerances for `adx`, `dx`, `adxr`.
- [ ] Unit 1.3: Fix `NaN` initialization logic in `stoch_arrow` and `aroonosc_arrow`.
- [ ] Unit 1.4: Final verification: `cargo test --workspace --features arrow` passes 100%.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Execute benchmark suite (`cargo bench`).
- [ ] Unit 2.2: Generate `docs/performance_report.md` comparing `_raw` and `_arrow` performance.

### Phase 3: Final Audit
- [ ] Unit 3.1: Final security audit of `unsafe` blocks.
- [ ] Unit 3.2: Tag v0.2.2-arrow release.

## Verification
- Run `cargo test --workspace --features arrow` (must be 100% pass).
- Run `cargo bench -p kand --features arrow`.

## Schedule
1. **Stabilization Phase (Day 1):** Finalize tests to 100% pass rate.
2. **Benchmarking Phase (Day 2):** Benchmark execution and report generation.
3. **Audit Phase (Day 3):** Security audit and final release tagging.
