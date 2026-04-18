# Plan: Performance Optimization (Buffer Management)

## Overview
This plan addresses the 16-30% overhead in Arrow-native wrappers by optimizing memory allocation and initialization patterns through optional buffer pooling and pre-allocation support.

## Problem Frame
The current `_arrow` variants perform a full heap allocation (`MutableBuffer::new`) and a sequential NaN-filling loop on every call. This plan introduces a thread-local buffer cache and extends the macro system to support caller-provided buffers.

## Requirements Trace
- **R1.** Implement `ThreadLocal` buffer cache behind `arrow-pooled` feature.
- **R2.** Extend macros to support optional pre-allocated buffers.
- **R3.** Optimize NaN initialization (bulk filling).
- **R4.** Quantify improvements via `criterion` benchmarks.

## Key Technical Decisions
- **Feature Gate:** `arrow-pooled` for opt-in pooling.
- **Pooling Strategy:** Use `RefCell<Vec<MutableBuffer>>` for thread-local storage of reusable buffers.

## Implementation Units

### Phase 1: Foundation & Cache Logic
- [ ] Unit 1.1: Research bulk-filling alternatives to standard `for` loop in Rust.
- [ ] Unit 1.2: Implement `ThreadLocal` buffer management module in `kand/src/helper/`.
- [ ] Unit 1.3: Update `kand/Cargo.toml` with `arrow-pooled` feature.

### Phase 2: Macro Refactor
- [ ] Unit 2.1: Update `kand_arrow_wrapper!` to utilize the buffer pool if feature is enabled.
- [ ] Unit 2.2: Add variants to macros that accept `&mut MutableBuffer` from the caller.

### Phase 3: Verification & Benchmarking
- [ ] Unit 3.1: Execute benchmarks comparing:
  - Legacy Slice
  - Current Arrow (Allocation per call)
  - Optimized Arrow (Pooled)
- [ ] Unit 3.2: Final security audit of buffer reuse (ensuring zero-over-write safety).

## Verification
- Benchmark: `cargo bench -p kand --bench bench_arrow --features arrow,arrow-pooled`.
- Test: `cargo test --workspace --features arrow,arrow-pooled`.
