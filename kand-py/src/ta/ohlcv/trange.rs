use kand::ohlcv::trange;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates True Range (TR) values for a series of price data.
...
///   >>> tr = kand.trange_inc(12.0, 9.0, 11.0)
///   ```
#[pyfunction]
#[pyo3(name = "trange_inc", signature = (high, low, prev_close))]
pub fn trange_inc_py(high: TAFloat, low: TAFloat, prev_close: TAFloat) -> PyResult<TAFloat> {
    trange::trange_inc(high, low, prev_close)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    trange_arrow,
    kand::ta::ohlcv::trange::trange_arrow,
    inputs: { high, low, close },
    params: {}
);
