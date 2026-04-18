# Plan: Performance Benchmarking and Audit

## Goal
Quantitatively verify the performance benefits of the Arrow zero-copy integration by benchmarking Arrow-native indicators against the legacy slice-based implementations.

## Methodology
- **Framework:** Use `criterion` for benchmarking.
- **Metrics:** Throughput (ops/sec), latency, and memory allocation overhead.
- **Datasets:** Synthetic OHLCV data of varying lengths (1k, 10k, 100k, 1M rows).
- **Environment:** Run benchmarks on the same hardware to ensure comparability.

## Benchmarking Plan

### 1. Instrumentation
- Enhance `kand/benches/bench_main.rs` to include Arrow variants for all 50+ indicators.
- Create a reusable benchmark helper that initializes both legacy `[TAFloat]` and Arrow `TAArrowArray` structures.

### 2. Execution Phases
- [ ] **Phase 1: Setup**
  - Consolidate benchmark code to use a unified `Criterion` configuration.
  - Ensure all 50+ indicators are registered in `bench_main.rs`.
- [ ] **Phase 2: Baseline Execution**
  - Run benchmarks on legacy `[TAFloat]` implementations.
  - Run benchmarks on `_arrow` variants.
- [ ] **Phase 3: Analysis**
  - Compare throughput and memory footprint (use `heaptrack` or `valgrind` if needed).
  - Report findings in `docs/performance_report.md`.

## Security & Reliability Audit
- Perform a final review of memory safety (buffer bounds, lifetime management of Arrow pointers).
- Validate the "Null-Reject" contract.

## Deliverables
- `docs/performance_report.md`: Comparative analysis of Arrow vs. Slice-based performance.
- `bench_main.rs`: Full benchmark suite.
