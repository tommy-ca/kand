use kand::ohlcv::cdl_hammer;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Detects Hammer candlestick patterns in price data.
...
///   >>> signal, body_avg = kand.cdl_hammer_inc(10.0, 11.0, 9.0, 10.5, 0.5, 14, 2.0)
///   ```
#[pyfunction]
#[pyo3(name = "cdl_hammer_inc", signature = (open, high, low, close, prev_body_avg, period, factor))]
pub fn cdl_hammer_inc_py(
    py: Python,
    open: TAFloat,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    prev_body_avg: TAFloat,
    period: usize,
    factor: TAFloat,
) -> PyResult<(TAInt, TAFloat)> {
    py.allow_threads(|| {
        cdl_hammer::cdl_hammer_inc(open, high, low, close, prev_body_avg, period, factor)
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    cdl_hammer_arrow,
    kand::ta::ohlcv::cdl_hammer::cdl_hammer_arrow,
    inputs: { open, high, low, close },
    params: { period: usize, factor: TAFloat },
    output_count: 2,
    output_types: { int, float }
);
