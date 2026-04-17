use kand::ohlcv::atr;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Average True Range (ATR) over NumPy arrays.
...
///   >>> atr = kand.atr_inc(
///   ...     15.0,  # high
///   ...     11.0,  # low
///   ...     12.0,  # prev_close
///   ...     3.0,   # prev_atr
///   ...     14     # period
///   ... )
///   ```
pub fn atr_inc_py(
    py: Python,
    high: TAFloat,
    low: TAFloat,
    prev_close: TAFloat,
    prev_atr: TAFloat,
    period: usize,
) -> PyResult<TAFloat> {
    py.allow_threads(|| atr::atr_inc(high, low, prev_close, prev_atr, period))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    atr_arrow,
    kand::ta::ohlcv::atr::atr_arrow,
    inputs: { high, low, close },
    params: { period: usize }
);
