use kand::ohlcv::adr;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates the Average Daily Range (ADR) for the entire price series.
...
///   >>> next_adr = kand.adr_inc(prev_adr, new_high, new_low, old_high, old_low, 14)
///   ```
#[pyfunction]
#[pyo3(name = "adr_inc", signature = (prev_adr, high, low, old_high, old_low, period))]
pub fn adr_inc_py(
    prev_adr: TAFloat,
    high: TAFloat,
    low: TAFloat,
    old_high: TAFloat,
    old_low: TAFloat,
    period: usize,
) -> PyResult<TAFloat> {
    adr::adr_inc(prev_adr, high, low, old_high, old_low, period)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    adr_arrow,
    kand::ta::ohlcv::adr::adr_arrow,
    inputs: { high, low },
    params: { period: usize }
);
