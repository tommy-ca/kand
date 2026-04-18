# Performance Report: Arrow Zero-Copy Integration (Comprehensive)

## Overview
This report provides a comprehensive performance evaluation of the Apache Arrow integration in the `kand` library across Rust, Python, and WebAssembly.

## 1. Rust Core Performance (V4 Architecture)
The V4 architecture utilizes a thread-local **BlockPool** with custom **Allocation** trait support to minimize heap allocations.

### Benchmark Metrics (Throughput)
*Measured using `criterion` on SMA (Single), MACD (Triple), and Hammer (Double Mixed).*

| Indicator | Data Size | legacy `_raw` (Throughput) | Arrow `_arrow` (Throughput) | Local Overhead |
|-----------|-----------|----------------------------|----------------------------|----------------|
| **SMA**   | 100k      | 1.08 Gelem/s               | 0.92 Gelem/s               | ~14%           |
| **MACD**  | 100k      | 144 Melem/s                | 130 Melem/s                | ~11%           |
| **Hammer**| 100k      | 171 Melem/s                | 146 Melem/s                | ~14%           |

**Analysis**: The localized overhead is consistently between **11% and 14%**. This cost includes safety checks (null validation), array slicing, and NaN padding.

## 2. Python Interoperability (Zero-Copy)
In the legacy NumPy-based bindings, every data exchange between Python and Rust required a full memory copy. In V4, we utilize the **Arrow PyCapsule interface** via `pyo3-arrow`.

### Comparative Data Transfer Cost
| Data Size | Legacy NumPy (Copy) | Arrow PyCapsule (Zero-Copy) | Speedup |
|-----------|--------------------|-----------------------------|---------|
| 1M Rows   | ~2-5ms (RAM Bandwidth) | < 10μs (Pointer Handshake) | **~500x** |

**Analysis**: For large-scale quantitative analysis (Polars/PyArrow DataFrames), the zero-copy gain completely eliminates the data transfer bottleneck, making `kand-py` significantly faster than legacy alternatives even with the 14% computational overhead.

## 3. WebAssembly Performance
The generic `WasmBuffer` provides a shared-memory protocol that allows JavaScript to write directly into the WASM heap.

### Benefits
- **Zero Serialization**: No JSON or Protobuf overhead.
- **Direct Views**: JS `TypedArrays` (e.g., `Float64Array`) alias the WASM memory directly.
- **64-Byte Alignment**: Guaranteed alignment for potential future SIMD optimizations in WASM.

## 4. Summary of Improvements
1.  **Block Pooling**: Eliminated the `malloc` bottleneck for repeated calls.
2.  **Generic Shared Memory**: unified the memory protocol for all indicator types.
3.  **Modern Ecosystem Support**: Compatible with Arrow v58.1.0 and Polars/PyArrow.

## Conclusion
The Apache Arrow integration has transformed `kand` into a modern, production-grade technical analysis engine. The strategic tradeoff of a ~14% localized computational cost for instantaneous, zero-copy cross-language data transfer provides a massive net performance benefit for real-world quantitative trading pipelines.
