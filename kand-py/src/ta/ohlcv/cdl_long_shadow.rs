use kand::ohlcv::cdl_long_shadow;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Detects Long Shadow candlestick patterns in price data.
...
///   >>> signal, body_avg = kand.cdl_long_shadow_inc(10.0, 11.0, 9.0, 10.5, 0.5, 14, 75.0)
///   ```
#[pyfunction]
#[pyo3(name = "cdl_long_shadow_inc", signature = (open, high, low, close, prev_body_avg, period, shadow_factor))]
pub fn cdl_long_shadow_inc_py(
    py: Python,
    open: TAFloat,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    prev_body_avg: TAFloat,
    period: usize,
    shadow_factor: TAFloat,
) -> PyResult<(TAInt, TAFloat)> {
    py.allow_threads(|| {
        cdl_long_shadow::cdl_long_shadow_inc(open, high, low, close, prev_body_avg, period, shadow_factor)
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    cdl_long_shadow_arrow,
    kand::ta::ohlcv::cdl_long_shadow::cdl_long_shadow_arrow,
    inputs: { open, high, low, close },
    params: { period: usize, shadow_factor: TAFloat },
    output_count: 2,
    output_types: { int, float }
);
