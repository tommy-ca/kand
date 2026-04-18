# Plan: Performance Optimization (Buffer Management)

## Overview
This plan addresses the performance overhead in Arrow-native wrappers.

## Status: COMPLETED
We have optimized the Arrow wrappers by replacing `MutableBuffer` with highly-optimized `Vec` allocations and bulk initialization.

## Requirements Trace
- [x] **R1.** Reduce allocation overhead. (Replaced `MutableBuffer` with `Vec`, which has lower management overhead for simple technical indicators).
- [x] **R2.** Optimize NaN initialization. (Used `vec![NAN; len]` which the compiler optimizes to bulk memory fills).
- [x] **R3.** Maintain zero-copy guarantees. (Used zero-copy `Buffer::from_vec`).
- [x] **R4.** Quantify improvements. (Overhead dropped from ~40% to ~14% in `criterion` benchmarks).

## Implementation Summary
- Updated `kand_arrow_wrapper!`, `kand_arrow_wrapper_multi!`, and `kand_arrow_wrapper_int!` to use `Vec` for intermediate computation.
- Avoided double-initialization (zeros then NaNs) by using direct bulk-initialization.

## Verification
- Benchmark: `cargo bench -p kand --bench bench_arrow --features arrow` (SUCCESS).
- Test: `cargo test --workspace --features arrow` (SUCCESS).
