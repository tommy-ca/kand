use kand::ohlcv::macd;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Moving Average Convergence Divergence (MACD) over a NumPy array.
...
///   >>> macd_line, signal_line, histogram = kand.macd_inc(
///   ...     100.0,  # current price
///   ...     95.0,   # previous fast EMA
///   ...     98.0,   # previous slow EMA
///   ...     -2.5,   # previous signal
///   ...     12,     # fast period
///   ...     26,     # slow period
///   ...     9       # signal period
///   ... )
///   ```
pub fn macd_inc_py(
    py: Python,
    price: TAFloat,
    prev_fast_ema: TAFloat,
    prev_slow_ema: TAFloat,
    prev_signal: TAFloat,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat)> {
    // Perform incremental MACD calculation while releasing the GIL
    py.allow_threads(|| {
        macd::macd_inc(
            price,
            prev_fast_ema,
            prev_slow_ema,
            prev_signal,
            fast_period,
            slow_period,
            signal_period,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    macd_arrow,
    kand::ta::ohlcv::macd::macd_arrow,
    inputs: { data },
    params: {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize
    },
    output_count: 5
);
