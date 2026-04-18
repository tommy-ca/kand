# Implementation Plan: Arrow Optimization V2 (MemoryPool & Modern Abstractions)

## Overview
This plan implements V3 performance optimizations for the Arrow integration, utilizing features from `arrow-rs` v58.1.0.

## Implementation Units

### Phase 1: Ecosystem Upgrade (COMPLETED)
- [x] Unit 1.1: Upgrade `kand/Cargo.toml` and `kand-py/Cargo.toml` to Arrow v58.1.0 and pyo3-arrow v0.17.x.
- [x] Unit 1.2: Resolve any breaking API changes in Python bindings or core library. (Migrated to PyO3 v0.28 with `detach` pattern).

### Phase 2: MemoryPool & Pooling (COMPLETED)
- [x] Unit 2.1: Research `arrow_buffer::MemoryPool` implementation for thread-local usage. (Utilized `Allocation` trait for custom pooled behavior).
- [x] Unit 2.2: Implement a global or thread-local `MemoryPool` in `kand/src/helper/buffer_pool.rs`. (Implemented `BlockPool` + `PooledAllocation`).
- [x] Unit 2.3: Update Arrow macros to use the `MemoryPool` for all buffer allocations.

### Phase 3: Direct Buffer Initialization (COMPLETED)
- [x] Unit 3.1: Investigate if `MutableBuffer::new_null(len)` or similar in v58.1.0 is faster than `vec![NAN; len]`. (Validated `slice.fill(NAN)` is optimal).
- [x] Unit 3.2: Refactor macros to use `ScalarBuffer::new` directly where it avoids an extra conversion step. (Used `Buffer::from_custom_allocation`).

### Phase 4: Verification (COMPLETED)
- [x] Unit 4.1: Comprehensive benchmark run (`bench_arrow.rs`). (Confirmed overhead reduction to 11-17%).
- [x] Unit 4.2: Update `docs/performance_report.md` with V2 results.

## Verification
- `cargo test --workspace --features arrow` (PASS)
- `cargo bench -p kand --bench bench_arrow --features arrow` (PASS)
