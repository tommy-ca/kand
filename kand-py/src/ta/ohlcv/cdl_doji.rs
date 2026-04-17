use kand::ohlcv::cdl_doji;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Detects Doji candlestick patterns in price data.
...
///   >>> signal = kand.cdl_doji_inc(10.0, 11.0, 9.8, 10.3, 5.0, 100.0)
///   ```
#[pyfunction]
#[pyo3(name = "cdl_doji_inc", signature = (open, high, low, close, body_percent, shadow_equal_percent))]
pub fn cdl_doji_inc_py(
    py: Python,
    open: TAFloat,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    body_percent: TAFloat,
    shadow_equal_percent: TAFloat,
) -> PyResult<TAInt> {
    py.allow_threads(|| {
        cdl_doji::cdl_doji_inc(open, high, low, close, body_percent, shadow_equal_percent)
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_int!(
    cdl_doji_arrow,
    kand::ta::ohlcv::cdl_doji::cdl_doji_arrow,
    inputs: { open, high, low, close },
    params: { body_percent: TAFloat, shadow_equal_percent: TAFloat }
);
