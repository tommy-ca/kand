# Implementation Plan: Phase 6 - Production Stateful Streaming

## Overview
This plan implements a production-ready stateful indicator framework that leverages vectorized Arrow operations for high-density streaming (thousands of assets simultaneously).

## Requirements Trace
- **R1. Stateful EMA**: Implement single-stream and vectorized multi-stream EMA.
- **R2. Stateful MACD**: Implement single-stream and vectorized multi-stream MACD.
- **R3. State Persistence**: Implement full `RecordBatch` serialization for `BatchSMA` and `BatchEMA`.
- **R4. Python/WASM Exposure**: Scale the `BatchIndicator` class exposure to Python and WASM.

## Key Technical Decisions
- **Vectorized State**: Hold the state for $N$ streams in contiguous Arrow-aligned buffers.
- **Persistence**: Use Arrow `RecordBatch` to store the internal state arrays, allowing zero-copy save/load.

## Implementation Units

### Phase 1: EMA & MACD Expansion
- [ ] Unit 1.1: Implement `StatefulEMA` and `BatchEMA` in `kand/src/ta/ohlcv/ema.rs`.
- [ ] Unit 1.2: Implement `StatefulMACD` and `BatchMACD` in `kand/src/ta/ohlcv/macd.rs`.
- [ ] Unit 1.3: Verify with TDD parity tests.

### Phase 2: Production Persistence
- [ ] Unit 2.1: Implement `to_record_batch` and `from_record_batch` for all stateful indicators.
- [ ] Unit 2.2: Add integration tests for state save/load cycles.

### Phase 3: Binding Scaling
- [ ] Unit 3.1: Expose `BatchEMA` and `BatchMACD` to Python bindings.
- [ ] Unit 3.2: Update WASM bindings with stateful wrappers.

## Verification
- `cargo test --workspace --features arrow`
- Benchmark: 10,000 parallel streams update performance.
