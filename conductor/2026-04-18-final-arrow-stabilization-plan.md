# Implementation Plan: Final Test Parity and Benchmarking (Phase 3)

## Overview
This plan focuses on achieving 100% test parity for the Arrow-integrated `kand` library by resolving the final 4 test failures and executing the performance benchmarking suite.

## Problem Frame
The current test suite has reached a 97% pass rate (122/126 passed). The remaining failures are:
1. **ADOSC (ADOSC_raw/ADOSC_arrow):** Discrepancy in A/D accumulation EMA startup.
2. **Stoch (Stoch_raw/Stoch_arrow):** Discrepancy in Stochastic Oscillator initialization timing.

## Requirements Trace
- **R1.** Achieve 100% test pass rate for all Arrow-native indicators.
- **R2.** Correct `adosc` and `stoch` EMA/initialization logic.
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Documentation of benchmark results.

## Implementation Units

### Phase 1: Surgical Test Parity (Final Sprint)
- [ ] Unit 1.1: Fix `adosc` initialization (EMA start-index parity).
- [ ] Unit 1.2: Fix `stoch` initialization (NaN padding and start-index).
- [ ] Unit 1.3: Final verification: `cargo test --workspace --features arrow` passes 100%.

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
