use kand::{TAFloat, ohlcv::sma};
#[cfg(feature = "arrow")]
use kand::ta::types::TAArrowArray;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
#[cfg(feature = "arrow")]
use pyo3_arrow::PyArray;
#[cfg(feature = "arrow")]
use std::sync::Arc;

/// Computes the Simple Moving Average (SMA) over a NumPy array.
...
/// )
/// .unwrap();
/// ```
pub fn sma_inc_py(
    prev_sma: TAFloat,
    new_price: TAFloat,
    old_price: TAFloat,
    period: usize,
) -> PyResult<TAFloat> {
    sma::sma_inc(prev_sma, new_price, old_price, period)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Computes the Simple Moving Average (SMA) over an Arrow array (zero-copy).
///
/// Args:
///     data: Input data as an Arrow array (via PyCapsule).
///     period: Window size for SMA calculation.
///
/// Returns:
///     An Arrow array containing the SMA values.
#[cfg(feature = "arrow")]
#[pyfunction]
#[pyo3(name = "sma_arrow", signature = (data, period))]
pub fn sma_arrow_py(
    py: Python,
    data: PyArray,
    period: usize,
) -> PyResult<PyArray> {
    let array = data.as_ref();
    let downcasted = array.as_any().downcast_ref::<TAArrowArray>()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyTypeError, _>("Expected compatible Arrow floating-point array"))?;

    let result = py.allow_threads(|| sma::sma_arrow(downcasted, period))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(PyArray::new(Arc::new(result), data.field().clone()))
}
