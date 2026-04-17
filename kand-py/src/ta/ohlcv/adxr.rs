use kand::ohlcv::adxr;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates the Average Directional Index Rating (ADXR) for the entire input array
...
///   >>> adxr, adx, plus_dm, minus_dm, tr = kand.adxr_inc(
///   ...     24.20, 23.85, 24.07, 23.72, 23.95, 25.0, 20.0, 0.5, 0.3, 1.2, 14)
///   ```
#[pyfunction]
#[pyo3(name = "adxr_inc", signature = (high, low, prev_high, prev_low, prev_close, prev_adx, prev_adx_period_ago, prev_smoothed_plus_dm, prev_smoothed_minus_dm, prev_smoothed_tr, period))]
pub fn adxr_inc_py(
    high: TAFloat,
    low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    prev_close: TAFloat,
    prev_adx: TAFloat,
    prev_adx_period_ago: TAFloat,
    prev_smoothed_plus_dm: TAFloat,
    prev_smoothed_minus_dm: TAFloat,
    prev_smoothed_tr: TAFloat,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat)> {
    adxr::adxr_inc(
        high,
        low,
        prev_high,
        prev_low,
        prev_close,
        prev_adx,
        prev_adx_period_ago,
        prev_smoothed_plus_dm,
        prev_smoothed_minus_dm,
        prev_smoothed_tr,
        period,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    adxr_arrow,
    kand::ta::ohlcv::adxr::adxr_arrow,
    inputs: { high, low, close },
    params: { period: usize },
    output_count: 5
);
