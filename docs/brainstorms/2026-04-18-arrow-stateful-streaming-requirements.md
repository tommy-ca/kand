---
date: 2026-04-18
topic: arrow-stateful-streaming-indicators
---

# Stateful Indicators with Arrow Streaming Support

## Problem Frame
The current `kand` library utilizes a purely functional approach for incremental updates (`_inc` variants), where state (e.g., `prev_sum`) must be managed explicitly by the caller. This becomes complex for high-density streaming applications (thousands of assets) and lacks standardized persistence.

The goal is to implement encapsulated, trait-based **Stateful Indicators** that leverage Apache Arrow for vectorized state updates and zero-copy persistence.

## Requirements
- **R1. Stateful Trait**: Define a standard trait (e.g., `Indicator`) for stateful computation.
- **R2. Vectorized State Updates**: Support a `BatchIndicator` pattern where state for $N$ independent assets is held in a single Arrow-backed structure and updated simultaneously using vectorized operations.
- **R3. Arrow Persistence**: State must be exportable to and importable from Arrow `RecordBatch` format.
- **R4. Zero-Copy Streaming**: Ensure that high-frequency updates to the state do not trigger unnecessary heap allocations.
- **R5. Multi-Language Parity**: Expose these stateful objects to Python (via `#[pyclass]`) and WASM.

## Success Criteria
- [ ] Implement `StatefulSMA` as a proof-of-concept for vectorized multi-stream updates.
- [ ] Demonstrate a 1000-stream parallel update benchmark.
- [ ] Successful save/load of indicator state via Arrow `RecordBatch`.

## Scope Boundaries
- **In Scope**: Trait definition, `RecordBatch` mapping, Vectorized SMA/EMA/MACD.
- **Out of Scope**: Distributed state management (this provides the primitive for it).

## Key Decisions
- **Trait-based Encapsulation**: Prioritize a standard interface over ad-hoc structs.
- **Vectorized Multi-State Support**: Design for high-density streaming (parallel assets) from the ground up.

## Dependencies / Assumptions
- Depends on `arrow-rs` v58.1.0 and existing `BlockPool` alignment.

## Outstanding Questions

### Deferred to Planning
- [Technical] How do we handle varying lookback requirements in a vectorized `BatchIndicator`?
- [Needs research] Performance delta between manual `Vec<State>` vs. unified `ArrowRecordBatch` for thousands of small states.

## Next Steps
→ /ce:plan for structured implementation of the `StatefulIndicator` framework.
