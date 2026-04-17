use kand::ohlcv::bop;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates the Balance of Power (BOP) indicator for a price series.
...
///   >>> output_bop = kand.bop_inc(10.0, 12.0, 8.0, 11.0)
///   ```
#[pyfunction]
#[pyo3(name = "bop_inc", signature = (open, high, low, close))]
pub fn bop_inc_py(
    open: TAFloat,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
) -> PyResult<TAFloat> {
    bop::bop_inc(open, high, low, close)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    bop_arrow,
    kand::ta::ohlcv::bop::bop_arrow,
    inputs: { open, high, low, close },
    params: {}
);
