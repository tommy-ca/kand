# High-Depth Audit: Arrow Zero-Copy & Streaming Architecture

## 1. Executive Summary
The V4 architecture of `kand` successfully delivers a high-performance, Arrow-native technische analysis engine. The current implementation of batch and stateful indicators is correct and demonstrates significant performance gains. However, several architectural improvements are suggested to reach "Enterprise-Grade" durability and scalability.

## 2. Issues & Observations

### 2.1 State Atomicity (MACD Example)
**Issue**: Multi-component indicators (like MACD, which uses three EMAs) are not currently atomic. If an update to the second EMA fails, the first EMA remains updated, while the third does not.
**Impact**: Continuous streaming may result in corrupted indicator states if errors are not handled by completely discarding the object.
**Suggestion**: Implement a "Transactional Update" or "Snapshot-Revert" pattern where state changes are only committed if all sub-operations succeed.

### 2.2 Persistence Schema
**Issue**: Scalar state (e.g., `period`, `cursor`, `count`) is currently stored in `RecordBatch` metadata.
**Impact**: While functional, metadata is often lost or ignored by standard Arrow tools (e.g., DataFusion, DuckDB) during join/merge operations.
**Suggestion**: Store scalars as 1-element constant columns in the `RecordBatch` for better interoperability and visibility.

### 2.3 Macro Coverage
**Issue**: Only SMA, EMA, and MACD have stateful/batch support (~4% of the library).
**Impact**: High manual implementation cost for the remaining 70+ indicators.
**Suggestion**: Extend the `kand_arrow_wrapper!` ecosystem to automatically generate `StatefulIndicator` and `BatchIndicator` implementations for all sliding-window based indicators.

### 2.4 Error Propagation in Python/WASM
**Observation**: Errors in the hot-path (`next_batch`) are caught and converted to `JsValue` or `PyValueError`.
**Recommendation**: Explicitly document that stateful objects should be considered "poisoned" if a `next` call returns an error.

## 3. Security Audit (Pointer Soundness)

### 3.1 `Buffer::from_custom_allocation`
- **Audit**: Verified that `PooledAllocation` correctly holds the `Block` (which owns the memory) until the Arrow `Buffer` is dropped.
- **Soundness**: The `thread_local!` cache ensures that blocks are never shared across threads while in the cache.
- **Recommendation**: Standardize on `NonZeroUsize` for lengths in the `BlockPool` to avoid edge-case allocations of zero bytes.

### 3.2 WASM Shared Memory
- **Audit**: `WasmBuffer` provides a safe way for JS to view Rust memory.
- **Risk**: JS views become "detached" on WASM memory growth.
- **Recommendation**: Add a versioning counter to `WasmBuffer` that increments on every `resize`. JS can check this counter to decide if it needs to re-instantiate its `TypedArray` views.

## 4. Next Steps
1.  **Refine Persistence**: Standardize on column-based scalar storage.
2.  **Automate Stateful Core**: Prioritize the development of a `kand_stateful_wrapper!` macro.
3.  **Stability**: Merge and tag `v0.2.3` with these refinements.
