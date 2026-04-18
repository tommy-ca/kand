# Implementation Plan: Arrow Integration Stabilization & Benchmarking (Rescheduled)

## Overview
This plan focuses on finalizing the stabilization of the Python (`kand-py`) bindings to resolve compilation errors and subsequently executing the performance benchmarking suite.

## Problem Frame
The Rust core is Arrow-native and stable. The Python bindings require targeted fixes for `MAType` conversion and error propagation to achieve successful compilation and type-safe interoperability. Once stabilized, we will move to performance benchmarking to validate the Arrow zero-copy throughput improvements.

## Requirements Trace
- **R1.** Finalize Python binding wrappers for indicators using `MAType` (ADOSC, BBands).
- **R2.** Ensure consistent Python return types (tuples for multi-output).
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Clean compilation and warning-free build.

## Key Technical Decisions
- **Consistency:** Python macro signatures must be 1:1 with core indicators.
- **Benchmarking:** Use `criterion` to compare slice-based (`_raw`) vs Arrow-native (`_arrow`) throughput.

## Implementation Units

### Phase 1: Python Binding Stabilization (High Priority)
- [ ] Unit 1.1: Fix `MAType` conversion in `adosc_py` and `bbands_py` (accept `u32`, convert to `MAType`).
- [ ] Unit 1.2: Correct return type pattern matching in `aroon_inc_py` and `aroonosc_inc_py` (using `Ok((...))`).
- [ ] Unit 1.3: Systematic removal of all `Signal` import warnings in core `kand`.
- [ ] Unit 1.4: Final verify: `cargo check -p kand-py --features arrow`.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Implement benchmark suite (`kand/benches/bench_main.rs`).
- [ ] Unit 2.2: Execute benchmark suite.
- [ ] Unit 2.3: Generate `docs/performance_report.md`.

### Phase 3: Final Audit
- [ ] Unit 3.1: Final security audit of `unsafe` blocks.
- [ ] Unit 3.2: Tag v0.2.2-arrow release.

## Verification
- Compile bindings: `cargo check -p kand-py --features arrow`.
- Benchmark: `cargo bench -p kand --features arrow`.
- Test: `cargo test --workspace --features arrow`.

## Rescheduled Timeline
1. **Binding Stabilization:** Finalize Python logic today.
2. **Benchmarking:** Execution and analysis scheduled for next session.
3. **Audit:** Final security audit and release tagging.
