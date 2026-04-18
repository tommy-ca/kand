---
date: 2026-04-18
topic: arrow-wrapper-performance-optimization
---

# Arrow Wrapper Performance Optimization (Requirements Draft)

## Problem Frame
Recent benchmarks show that Arrow-native wrappers (`_arrow` variants) in `kand` incur a 16-30% performance overhead compared to raw slice-based implementations. This overhead is primarily driven by:
1.  **High Allocation Frequency**: A new `MutableBuffer` is allocated on every function call.
2.  **Sequential Initialization**: A standard loop is used to fill the output buffer with `NaN` values before computation.

While zero-copy benefits between languages (Python/WASM) outweigh this overhead for large datasets, we aim to minimize it for high-frequency internal Rust usage.

## Requirements
- R1. **Allocation Reduction**: Minimize heap allocations during `_arrow` calls.
- R2. **Optimized Padding**: Implement high-performance buffer initialization (e.g., bulk filling or SIMD).
- R3. **API Consistency**: Optimizations must not break existing Arrow PyCapsule or WASM shared-memory contracts.
- R4. **Thread Safety**: Buffer pooling or reuse must be thread-safe (e.g., via `ThreadLocal` or global atomic pools).

## Success Criteria
- [ ] Reduce `_arrow` variant overhead to <10% compared to `_raw` for 10k+ element datasets.
- [ ] Maintain 100% test parity and zero-copy guarantees.
- [ ] Successful execution of comparative benchmarks.

## Scope Boundaries
- Out of scope: Changing the fundamental mathematical logic of indicators.
- Out of scope: Refactoring non-Arrow Python/WASM bindings.

## Key Decisions
- **Buffer Reuse**: We will prioritize a `ThreadLocal` buffer cache to avoid global lock contention.
- **Pre-allocation Support**: The Arrow macros will be extended to optionally accept an existing buffer, allowing power users to manage memory manually.

## Outstanding Questions

### Resolve Before Planning
- [User decision] Should buffer pooling be enabled by default, or behind a feature gate (e.g., `arrow-optimized`)?

### Deferred to Planning
- [Technical] What is the impact of buffer reuse on memory fragmentation for varying dataset sizes?
- [Needs research] Does `arrow-rs` provide a built-in "fast-fill" mechanism for `NaN` values that out-performs a standard `for` loop?

## Next Steps
→ Finalize Requirements -> Review -> Implementation Plan.
