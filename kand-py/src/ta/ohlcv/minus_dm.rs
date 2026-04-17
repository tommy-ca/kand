use kand::ohlcv::minus_dm;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates Minus Directional Movement (-DM) for a price series.
...
///   ```
#[pyfunction]
#[pyo3(name = "minus_dm_inc", signature = (high, prev_high, low, prev_low, prev_minus_dm, period))]
pub fn minus_dm_inc_py(
    high: TAFloat,
    prev_high: TAFloat,
    low: TAFloat,
    prev_low: TAFloat,
    prev_minus_dm: TAFloat,
    period: usize,
) -> PyResult<TAFloat> {
    minus_dm::minus_dm_inc(high, prev_high, low, prev_low, prev_minus_dm, period)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    minus_dm_arrow,
    kand::ta::ohlcv::minus_dm::minus_dm_arrow,
    inputs: { high, low },
    params: { period: usize }
);
