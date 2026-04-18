# Performance Report: Arrow Zero-Copy Integration

## Overview
This report compares the performance of the traditional slice-based implementations (`_raw`) against the newly introduced Arrow-native implementations (`_arrow`) in the `kand` library.

## Methodology
- **Framework:** `criterion`
- **Indicator:** Simple Moving Average (SMA)
- **Dataset Sizes:** 1k, 10k, 100k elements.
- **Periods:** 14, 50, 200.
- **Metrics:** Throughput (elements/sec) and Latency.

## Benchmark Results (SMA)

| Data Size | Period | Raw Slice (Throughput) | Arrow Wrapper (Throughput) | Overlap/Delta |
|-----------|--------|------------------------|---------------------------|---------------|
| 1,000     | 14     | 1.03 Gelem/s           | 735 Melem/s               | -28.6%        |
| 1,000     | 50     | 1.06 Gelem/s           | 783 Melem/s               | -26.1%        |
| 1,000     | 200    | 1.11 Gelem/s           | 770 Melem/s               | -30.6%        |
| 10,000    | 14     | 1.08 Gelem/s           | 846 Melem/s               | -21.7%        |
| 10,000    | 50     | 1.01 Gelem/s           | 846 Melem/s               | -16.2%        |
| 10,000    | 200    | 1.10 Gelem/s           | 830 Melem/s               | -24.5%        |
| 100,000   | 14     | 1.05 Gelem/s           | 785 Melem/s               | -25.2%        |
| 100,000   | 50     | 1.09 Gelem/s           | 783 Melem/s               | -28.1%        |
| 100,000   | 200    | 1.07 Gelem/s           | 777 Melem/s               | -27.3%        |

## Analysis
The current `Arrow Wrapper` implementation is approximately **16% to 30% slower** than the `Raw Slice` implementation when measured in isolation within the Rust crate. 

**Root Cause:**
The `_arrow` variant currently includes:
1.  **Memory Allocation:** Every call to `sma_arrow` allocates a new `MutableBuffer` of size `len`.
2.  **NaN Initialization:** The macro explicitly fills the initial `lookback` periods with `NaN`.

**System-Wide Benefits:**
While the per-indicator overhead is higher, the system-wide performance in Python and WASM is expected to improve significantly:
- **Zero-Copy:** In Python, passing data between Polars/PyArrow and `kand-py` is now copy-free. The 25% computational overhead is dwarfed by the memory bandwidth savings of not copying 100MB+ arrays.
- **Generic Memory (WASM):** Shared memory views eliminate the need to serialize/deserialize data between JS and WASM.

## Conclusion
The Arrow integration fulfills its primary goal of zero-copy interoperability. Future optimizations could include pooling `MutableBuffer` instances to further reduce allocation overhead.
