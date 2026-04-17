use kand::ohlcv::cdl_marubozu;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Identifies Marubozu candlestick patterns in price data.
...
///   >>> signal, body_avg = kand.cdl_marubozu_inc(10.0, 10.5, 9.8, 10.4, 10.2, 14, 5.0)
///   ```
#[pyfunction]
#[pyo3(name = "cdl_marubozu_inc", signature = (open, high, low, close, prev_body_avg, period, shadow_percent))]
pub fn cdl_marubozu_inc_py(
    py: Python,
    open: TAFloat,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    prev_body_avg: TAFloat,
    period: usize,
    shadow_percent: TAFloat,
) -> PyResult<(TAInt, TAFloat)> {
    py.allow_threads(|| {
        cdl_marubozu::cdl_marubozu_inc(open, high, low, close, prev_body_avg, period, shadow_percent)
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    cdl_marubozu_arrow,
    kand::ta::ohlcv::cdl_marubozu::cdl_marubozu_arrow,
    inputs: { open, high, low, close },
    params: { period: usize, shadow_percent: TAFloat },
    output_count: 2,
    output_types: { int, float }
);
