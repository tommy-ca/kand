use kand::ohlcv::aroon;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates the Aroon indicator for a price series.
...
///   >>> aroon_up, aroon_down, new_high, new_low, days_high, days_low = kand.aroon_inc(
///   ...     15.0, 12.0, 14.0, 11.0, 2, 1, 14)
///   ```
#[pyfunction]
#[pyo3(name = "aroon_inc", signature = (high, low, prev_high, prev_low, days_since_high, days_since_low, period))]
pub fn aroon_inc_py(
    high: TAFloat,
    low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    days_since_high: usize,
    days_since_low: usize,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat, usize, usize)> {
    aroon::aroon_inc(
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
crate::kand_py_arrow_wrapper_multi!(
    aroon_arrow,
    kand::ta::ohlcv::aroon::aroon_arrow,
    inputs: { high, low },
    params: { period: usize },
    output_count: 2
);
