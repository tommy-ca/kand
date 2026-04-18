---
date: 2026-04-18
topic: arrow-buffer-optimization-v2
---

# Arrow Buffer Optimization v2: MemoryPool and ScalarBuffer

## Problem Frame
While the previous optimization (using `Vec` and `Buffer::from_vec`) reduced overhead to ~14%, we seek to further minimize it by leveraging the latest features in `arrow-buffer` v58.1.0, specifically the `MemoryPool` trait and optimized `ScalarBuffer` abstractions.

## Requirements
- R1. **Dependency Upgrade**: Modernize to `arrow-rs` v58.1.0 and `pyo3-arrow` v0.17.x.
- R2. **Buffer Pooling (MemoryPool)**: Implement or utilize an existing `MemoryPool` to eliminate per-call heap allocations.
- R3. **Direct ScalarBuffer usage**: Investigate if bypassing `Vec` and using `ScalarBuffer` directly (via `MutableBuffer`) provides better cache locality or initialization performance.
- R4. **Advanced Initialization**: Research and implement bulk-filling of NaNs using SIMD if not already optimized by the compiler in v58.1.0.

## Success Criteria
- [ ] Reduce `_arrow` variant overhead to <5% for large datasets.
- [ ] Clean compilation on latest Arrow/PyO3-Arrow ecosystem.

## Scope Boundaries
- Out of scope: Breaking the generic `WasmBuffer` contract.
- Out of scope: Changes to non-performance-critical metadata handling.

## Key Decisions
- **Pooling**: We will evaluate `TrackingMemoryPool` or a custom `ThreadLocalPool` that adheres to the `MemoryPool` trait.
- **Precision**: Maintain `f32`/`f64` feature parity.

## Next Steps
→ Upgrade dependencies -> Research MemoryPool implementation details -> Update Macros.
