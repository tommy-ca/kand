use kand::ohlcv::plus_dm;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Plus Directional Movement (+DM) over NumPy arrays.
...
///   ```
#[pyfunction]
#[pyo3(name = "plus_dm_inc", signature = (high, prev_high, low, prev_low, prev_plus_dm, period))]
pub fn plus_dm_inc_py(
    high: TAFloat,
    prev_high: TAFloat,
    low: TAFloat,
    prev_low: TAFloat,
    prev_plus_dm: TAFloat,
    period: usize,
) -> PyResult<TAFloat> {
    plus_dm::plus_dm_inc(high, prev_high, low, prev_low, prev_plus_dm, period)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    plus_dm_arrow,
    kand::ta::ohlcv::plus_dm::plus_dm_arrow,
    inputs: { high, low },
    params: { period: usize }
);
