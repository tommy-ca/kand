# Implementation Plan: Production Readiness & Stateful Arrow Framework

## Overview
This plan focuses on finalizing the production audit of the `feat/arrow-zero-copy` branch and implementing a new `StatefulIndicator` framework that leverages vectorized Arrow operations for high-density streaming.

## Problem Frame
The current Arrow integration is optimal for batch processing but lacks an encapsulated state management system for streaming. We also need a final safety audit to ensure production readiness.

## Requirements Trace
- **R1. Production Readiness Audit**: Final review of `BlockPool` and `unsafe` blocks.
- **R2. Stateful Trait**: Define `ta::Indicator` trait in `kand`.
- **R3. Vectorized Multi-State**: Implement `BatchSMA` utilizing `TAArrowArray` for internal state.
- **R4. Python/WASM Exports**: Expose stateful objects to bindings.

## Key Technical Decisions
- **`Indicator` Trait**:
    ```rust
    pub trait Indicator {
        type Input;
        type Output;
        fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError>;
    }
    ```
- **Vectorized `BatchIndicator`**:
    ```rust
    pub struct BatchSMA {
        period: usize,
        prev_sums: MutableBuffer, // Vectorized state for N streams
    }
    ```

## Implementation Units

### Phase 1: Production Audit & Documentation
- [ ] Unit 1.1: Formal memory safety audit of `Buffer::from_custom_allocation`.
- [ ] Unit 1.2: Standardize all `_arrow` doc comments and add "Streaming" section to `README.md`.

### Phase 2: Stateful Indicator Core
- [ ] Unit 2.1: Define `Indicator` and `BatchIndicator` traits in `kand/src/ta/mod.rs`.
- [ ] Unit 2.2: Implement `StatefulSMA` (single stream) and `BatchSMA` (vectorized multi-stream).
- [ ] Unit 2.3: Add `to_record_batch` and `from_record_batch` for persistence.

### Phase 3: Bindings & Benchmarking
- [ ] Unit 3.1: Expose `BatchSMA` to `kand-py` as a `#[pyclass]`.
- [ ] Unit 3.2: Benchmark 10,000 parallel SMA streams using vectorized vs. serial updates.

## Verification
- `cargo test --workspace --features arrow`
- Benchmark: Verify >5x speedup for vectorized multi-stream updates.
