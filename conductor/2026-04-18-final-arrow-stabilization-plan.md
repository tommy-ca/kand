# Implementation Plan: Arrow Integration Stabilization & Benchmarking

## Overview
This plan addresses the final stabilization of the `kand` bindings (Python/WASM) and the execution of the performance benchmarking suite.

## Problem Frame
While the Rust core is Arrow-native and stable, the Python and WASM bindings require final adjustments to correctly interface with the updated multi-output and heterogeneous Arrow macro signatures. Once stabilized, we must quantitatively validate the performance gains.

## Requirements Trace
- **R1.** Stabilize `kand-py` macro wrappers for complex/multi-output indicators.
- **R2.** Fix `MAType` and type-mismatch issues in WASM/Python bindings.
- **R3.** Formal Performance Benchmarking (Slice-based vs. Arrow-native).
- **R4.** Documentation of benchmark results.

## Key Technical Decisions
- **Consistency:** Python macro signatures must be 1:1 with core indicators.
- **Benchmarking:** Use `criterion` to compare slice-based (`_raw`) vs Arrow-native (`_arrow`) throughput.

## Implementation Units

### Phase 1: Binding Stabilization
- [ ] Unit 1.1: Fix `kand-py` macro mismatches (tuple return types, `MAType` handling).
- [ ] Unit 1.2: Stabilize `kand-wasm` for Batch 7 indicators.
- [ ] Unit 1.3: Verify `kand-py` and `kand-wasm` compilation with `arrow` feature.

### Phase 2: Performance Benchmarking
- [ ] Unit 2.1: Implement benchmark suite (`kand/benches/bench_main.rs`).
  - Must include: `sma`, `ema`, `macd`, `rsi`, `bbands`, and `cdl_hammer` (representing different complexity classes).
- [ ] Unit 2.2: Execute benchmark suite on target environment.
- [ ] Unit 2.3: Generate `docs/performance_report.md`.

### Phase 3: Final Audit
- [ ] Unit 3.1: Final security audit of `unsafe` blocks in `WasmBuffer` and macro expansion.
- [ ] Unit 3.2: Tag v0.2.2-arrow release.

## Verification
- Compile bindings: `cargo check -p kand-py --features arrow` and `cargo check -p kand-wasm`.
- Benchmark: `cargo bench -p kand --features arrow`.
- Test: `cargo test --workspace --features arrow`.
