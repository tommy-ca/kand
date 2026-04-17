use kand::ohlcv::aroonosc;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates Aroon Oscillator values for an entire price series.
...
///   >>> aroonosc, high, low, days_high, days_low = kand.aroonosc_inc(
///   ...     10.0, 9.0, 11.0, 10.0, 1, 2, 14)
///   ```
#[pyfunction]
#[pyo3(name = "aroonosc_inc", signature = (high, low, prev_high, prev_low, days_since_high, days_since_low, period))]
pub fn aroonosc_inc_py(
    high: TAFloat,
    low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    days_since_high: usize,
    days_since_low: usize,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, usize, usize)> {
    aroonosc::aroonosc_inc(
        high,
        low,
        prev_high,
        prev_low,
        days_since_high,
        days_since_low,
        period,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    aroonosc_arrow,
    kand::ta::ohlcv::aroonosc::aroonosc_arrow,
    inputs: { high, low },
    params: { period: usize }
);
