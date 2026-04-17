use kand::ohlcv::rsi;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Relative Strength Index (RSI) over NumPy arrays.
...
///   >>> rsi, avg_gain, avg_loss = kand.rsi_inc(45.42, 45.10, 0.24, 0.14, 14)
///   ```
pub fn rsi_inc_py(
    current_price: TAFloat,
    prev_price: TAFloat,
    prev_avg_gain: TAFloat,
    prev_avg_loss: TAFloat,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat)> {
    rsi::rsi_inc(
        current_price,
        prev_price,
        prev_avg_gain,
        prev_avg_loss,
        period,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    rsi_arrow,
    kand::ta::ohlcv::rsi::rsi_arrow,
    inputs: { prices },
    params: { period: usize },
    output_count: 3
);
