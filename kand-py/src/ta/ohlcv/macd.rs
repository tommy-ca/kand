use kand::{TAFloat, ohlcv::macd};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Moving Average Convergence Divergence (MACD) over a NumPy array.
///
/// MACD is a trend-following momentum indicator that shows the relationship between two moving averages
/// of an asset's price. It consists of three components:
/// - MACD Line: Difference between fast and slow EMAs
/// - Signal Line: EMA of the MACD line
/// - Histogram: Difference between MACD line and signal line
///
/// Args:
///   data: Input price data as a 1-D NumPy array of type `TAFloat`.
///   fast_period: Period for fast EMA calculation (typically 12).
///   slow_period: Period for slow EMA calculation (typically 26).
///   signal_period: Period for signal line calculation (typically 9).
///
/// Returns:
///   A tuple of three 1-D NumPy arrays containing:
///   - MACD line values
///   - Signal line values
///   - MACD histogram values
///   Each array has the same length as the input, with initial elements containing NaN values.
///
/// Examples:
///   ```python
///   >>> import numpy as np
///   >>> import kand
///   >>> data = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
///   >>> macd_line, signal_line, histogram = kand.macd(data, 2, 3, 2)
///   ```
#[pyfunction]
#[pyo3(name = "macd", signature = (data, fast_period, slow_period, signal_period))]
pub fn macd_py(
    py: Python,
    data: PyReadonlyArray1<TAFloat>,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> PyResult<(
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
)> {
    // Convert the input NumPy array to a Rust slice
    let input = data.as_slice()?;
    let len = input.len();

    // Create output arrays using vec
    let mut macd_line = vec![0.0; len];
    let mut signal_line = vec![0.0; len];
    let mut histogram = vec![0.0; len];

    // Perform MACD calculation while releasing the GIL
    py.detach(|| {
        macd::macd(
            input,
            fast_period,
            slow_period,
            signal_period,
            macd_line.as_mut_slice(),
            signal_line.as_mut_slice(),
            histogram.as_mut_slice(),
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    // Convert output arrays to Python objects
    Ok((
        macd_line.into_pyarray(py).into(),
        signal_line.into_pyarray(py).into(),
        histogram.into_pyarray(py).into(),
    ))
}

/// Computes the latest MACD values incrementally from previous state.
///
/// This function provides an efficient way to calculate MACD for streaming data by using
/// previous EMA values instead of recalculating the entire series.
///
/// Args:
///
///   price: Current price value as `TAFloat`.
///   prev_fast_ema: Previous fast EMA value as `TAFloat`.
///   prev_slow_ema: Previous slow EMA value as `TAFloat`.
///   prev_signal: Previous signal line value as `TAFloat`.
///   fast_period: Period for fast EMA calculation (typically 12).
///   slow_period: Period for slow EMA calculation (typically 26).
///   signal_period: Period for signal line calculation (typically 9).
///
/// Returns:
///   A tuple of three values:
///   - MACD line value
///   - Signal line value
///   - MACD histogram value
///
/// Examples:
///   ```python
///   >>> import kand
///   >>> macd_line, signal_line, histogram = kand.macd_inc(
///   ...     100.0,  # current price
///   ...     95.0,   # previous fast EMA
///   ...     98.0,   # previous slow EMA
///   ...     -2.5,   # previous signal
///   ...     12,     # fast period
///   ...     26,     # slow period
///   ...     9       # signal period
///   ... )
///   ```
#[pyfunction]
#[pyo3(name = "macd_inc", signature = (price, prev_fast_ema, prev_slow_ema, prev_signal, fast_period, slow_period, signal_period))]
pub fn macd_inc_py(
    py: Python,
    price: TAFloat,
    prev_fast_ema: TAFloat,
    prev_slow_ema: TAFloat,
    prev_signal: TAFloat,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat)> {
    // Perform incremental MACD calculation while releasing the GIL
    py.detach(|| {
        macd::macd_inc(
            price,
            prev_fast_ema,
            prev_slow_ema,
            prev_signal,
            fast_period,
            slow_period,
            signal_period,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    macd_arrow,
    kand::ta::ohlcv::macd::macd_arrow,
    inputs: { data },
    params: {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize
    },
    output_count: 3
);

#[cfg(feature = "arrow")]
#[pyclass(name = "BatchMACD")]
pub struct BatchMacdPy {
    inner: kand::ta::ohlcv::macd::BatchMACD,
}

#[cfg(feature = "arrow")]
#[pymethods]
impl BatchMacdPy {
    #[new]
    #[pyo3(signature = (fast_period, slow_period, signal_period, num_streams))]
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        num_streams: usize,
    ) -> PyResult<Self> {
        let inner = kand::ta::ohlcv::macd::BatchMACD::new(
            fast_period,
            slow_period,
            signal_period,
            num_streams,
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    #[pyo3(signature = (input))]
    pub fn next_batch(
        &mut self,
        py: Python,
        input: pyo3_arrow::PyArray,
    ) -> PyResult<(
        pyo3_arrow::PyArray,
        pyo3_arrow::PyArray,
        pyo3_arrow::PyArray,
    )> {
        use arrow::array::Array;
        use kand::ta::traits::BatchIndicator;
        use kand::ta::types::TAArrowArray;
        use std::sync::Arc;

        let input_arrow = input
            .as_ref()
            .as_any()
            .downcast_ref::<TAArrowArray>()
            .ok_or_else(|| {
                pyo3::exceptions::PyTypeError::new_err(
                    "Expected compatible Arrow floating-point array",
                )
            })?;

        let (m, s, h) = py
            .detach(|| self.inner.next_batch(input_arrow.clone()))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let m_field = Arc::new(arrow::datatypes::Field::new(
            "",
            m.data_type().clone(),
            true,
        ));
        let s_field = Arc::new(arrow::datatypes::Field::new(
            "",
            s.data_type().clone(),
            true,
        ));
        let h_field = Arc::new(arrow::datatypes::Field::new(
            "",
            h.data_type().clone(),
            true,
        ));

        Ok((
            pyo3_arrow::PyArray::new(Arc::new(m), m_field),
            pyo3_arrow::PyArray::new(Arc::new(s), s_field),
            pyo3_arrow::PyArray::new(Arc::new(h), h_field),
        ))
    }
}
