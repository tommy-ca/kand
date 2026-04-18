use kand::{TAFloat, ohlcv::ema};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Exponential Moving Average (EMA) over a NumPy array.
///
/// The Exponential Moving Average is calculated by applying more weight to recent prices
/// via a smoothing factor k. Each value is calculated as:
/// EMA = Price * k + EMA(previous) * (1 - k)
/// where k is typically 2/(period+1).
///
/// Args:
///   data: Input data as a 1-D NumPy array of type `TAFloat`.
///   period: Window size for EMA calculation. Must be positive and less than input length.
///   k: Optional custom smoothing factor. If None, uses default k = 2/(period+1).
///
/// Returns:
///   A new 1-D NumPy array containing the EMA values. The array has the same length as the input,
///   with the first `period-1` elements containing NaN values.
///
/// Examples:
///   ```python
///   >>> import numpy as np
///   >>> import kand
///   >>> data = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
///   >>> result = kand.ema(data, 3)
///   >>> print(result)
///   [nan, nan, 2.0, 3.0, 4.2]
///   ```
#[pyfunction]
#[pyo3(name = "ema", signature = (data, period, k=None))]
pub fn ema_py(
    py: Python,
    data: PyReadonlyArray1<TAFloat>,
    period: usize,
    k: Option<TAFloat>,
) -> PyResult<Py<PyArray1<TAFloat>>> {
    // Convert the input NumPy array to a Rust slice.
    let input = data.as_slice()?;
    let len = input.len();

    // Create a new output array using vec
    let mut output = vec![0.0; len];

    // Perform the EMA calculation while releasing the GIL to allow other Python threads to run.
    py.detach(|| ema::ema(input, period, k, output.as_mut_slice()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    // Convert the output array to a Python object
    Ok(output.into_pyarray(py).into())
}

/// Computes the latest EMA value incrementally.
///
/// This function provides an efficient way to calculate EMA values for new data without
/// reprocessing the entire dataset.
///
/// Args:
///
///   price: Current period's price value as `TAFloat`.
///   prev_ema: Previous period's EMA value as `TAFloat`.
///   period: Window size for EMA calculation. Must be >= 2.
///   k: Optional custom smoothing factor. If None, uses default k = 2/(period+1).
///
/// Returns:
///   The new EMA value as `TAFloat`.
///
/// Examples:
///   ```python
///   >>> import kand
///   >>> current_price = 15.0
///   >>> prev_ema = 14.5
///   >>> period = 14
///   >>> new_ema = kand.ema_inc(current_price, prev_ema, period)
///   ```
#[pyfunction]
#[pyo3(name = "ema_inc", signature = (price, prev_ema, period, k=None))]
pub fn ema_inc_py(
    py: Python,
    price: TAFloat,
    prev_ema: TAFloat,
    period: usize,
    k: Option<TAFloat>,
) -> PyResult<TAFloat> {
    py.detach(|| {
        ema::ema_inc(price, prev_ema, period, k)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    })
}


// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    ema_arrow,
    kand::ta::ohlcv::ema::ema_arrow,
    inputs: { input_prices },
    params: { opt_period: usize, opt_k: Option<f64> }
);

#[cfg(feature = "arrow")]
#[pyclass(name = "BatchEMA")]
pub struct BatchEMA_py {
    inner: kand::ta::ohlcv::ema::BatchEMA,
}

#[cfg(feature = "arrow")]
#[pymethods]
impl BatchEMA_py {
    #[new]
    #[pyo3(signature = (period, num_streams, opt_k=None))]
    pub fn new(period: usize, num_streams: usize, opt_k: Option<f64>) -> PyResult<Self> {
        let inner = kand::ta::ohlcv::ema::BatchEMA::new(period, num_streams, opt_k)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    #[pyo3(signature = (input))]
    pub fn next_batch(&mut self, py: Python, input: pyo3_arrow::PyArray) -> PyResult<pyo3_arrow::PyArray> {
        use std::sync::Arc;
        use arrow::array::Array;
        use kand::ta::traits::BatchIndicator;
        use kand::ta::types::TAArrowArray;

        let input_arrow = input.as_ref()
            .as_any()
            .downcast_ref::<TAArrowArray>()
            .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Expected compatible Arrow floating-point array"))?;

        let result = py.detach(|| self.inner.next_batch(input_arrow.clone()))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let field = Arc::new(arrow::datatypes::Field::new("", result.data_type().clone(), true));
        Ok(pyo3_arrow::PyArray::new(Arc::new(result), field))
    }
}
