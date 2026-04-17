use kand::ohlcv::wma;
use pyo3::prelude::*;

/// Computes the Weighted Moving Average (WMA) over a NumPy array.
...
///   >>> window = [5.0, 4.0, 3.0]
///   >>> wma = kand.wma_inc(window, 3)
///   ```
#[pyfunction]
#[pyo3(name = "wma_inc", signature = (window, period))]
pub fn wma_inc_py(
    py: Python,
    window: Vec<TAFloat>,
    period: usize,
) -> PyResult<TAFloat> {
    py.allow_threads(|| wma::wma_inc(&window, period))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    wma_arrow,
    kand::ta::ohlcv::wma::wma_arrow,
    inputs: { data },
    params: { period: usize }
);
