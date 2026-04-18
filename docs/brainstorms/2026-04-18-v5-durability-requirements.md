---
date: 2026-04-18
topic: v5-enterprise-durability-requirements
---

# V5 Enterprise Durability Requirements: Columnar Persistence & Atomic State

## Problem Frame
The V4 architecture successfully implemented high-performance Arrow streaming, but the audit identified two critical reliability gaps:
1. **Metadata Vulnerability**: Persistence relies on Arrow metadata, which is often stripped by intermediate processing tools (joins, filters).
2. **State Corruption**: Multi-component indicators (MACD) can reach inconsistent internal states if sub-updates partially fail.
3. **Manual Overhead**: Stateful/Batch implementation requires significant boilerplate per indicator.

## Requirements
- **R1. Prefixed Columnar Persistence**: Store all configuration (period, multiplier) and transient state (cursor, count) as 1-element constant columns in the `RecordBatch`. Use a `__kand_` prefix (e.g., `__kand_period`) to avoid collisions with user data.
- **R2. Explicit Commit Pattern**: multi-component indicators must implement updates such that internal state is only modified if all sub-component operations succeed.
- **R3. Universal Indicator Macro**: Refactor the macro ecosystem into a single `kand_indicator!` macro that generates the full functional and stateful suite (_raw, safe, _arrow, Stateful, Batch) from a unified declaration.
- **R4. Poisoning Protection**: Public stateful APIs must explicitly detect and return `KandError::PoisonedState` if a previous operation failed.

## Success Criteria
- [ ] `BatchSMA` and `BatchEMA` utilize columnar persistence.
- [ ] `StatefulMACD` demonstrates atomicity in unit tests (rollback on error).
- [ ] `kand_indicator!` successfully generates a fully functional RSI indicator.
- [ ] 0 data loss when passing a saved state `RecordBatch` through a `polars` filter operation.

## Scope Boundaries
- **In Scope**: Refactoring existing SMA/EMA/MACD, developing the universal macro, updating persistence schema.
- **Out of Scope**: Distributed state backends (this provides the reliable local primitive).

## Key Decisions
- **Prefixed Column Scalars**: Standardized on `__kand_` prefix for better tool interoperability.
- **Explicit Commit Pattern**: Prioritize data integrity over zero-cost implementation.
- **Universal Indicator Macro**: Consolidate all variant generation into a single source-of-truth macro.

## Outstanding Questions

### Deferred to Planning
- [Technical] How does `kand_indicator!` handle indicators with non-standard state requirements (e.g., KAMA's adaptive speed)?
- [Needs research] Overhead of 1-element columns vs metadata in massive batch persistence scenarios.

## Next Steps
→ /ce:plan for implementation of the V5 Durability framework.
