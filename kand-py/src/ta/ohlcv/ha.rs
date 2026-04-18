use kand::{TAFloat, ohlcv::ha};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates Heikin-Ashi candlestick values from OHLC price data.
///
/// Args:
///   open: Array of opening prices as a 1-D NumPy array of type `TAFloat`.
///   high: Array of high prices as a 1-D NumPy array of type `TAFloat`.
///   low: Array of low prices as a 1-D NumPy array of type `TAFloat`.
///   close: Array of closing prices as a 1-D NumPy array of type `TAFloat`.
///
/// Returns:
///   A tuple of four 1-D NumPy arrays containing the HA open, HA high, HA low and HA close values.
///
/// Examples:
///   ```python
///   >>> import numpy as np
///   >>> import kand
///   >>> open = np.array([10.0, 10.5, 11.2])
///   >>> high = np.array([11.0, 11.5, 11.8])
///   >>> low = np.array([9.5, 10.2, 10.8])
///   >>> close = np.array([10.8, 11.3, 11.5])
///   >>> ha_open, ha_high, ha_low, ha_close = kand.ha(open, high, low, close)
///   ```
#[pyfunction]
#[pyo3(name = "ha", signature = (open, high, low, close))]
pub fn ha_py(
    py: Python,
    open: PyReadonlyArray1<TAFloat>,
    high: PyReadonlyArray1<TAFloat>,
    low: PyReadonlyArray1<TAFloat>,
    close: PyReadonlyArray1<TAFloat>,
) -> PyResult<(
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
)> {
    let input_open = open.as_slice()?;
    let input_high = high.as_slice()?;
    let input_low = low.as_slice()?;
    let input_close = close.as_slice()?;
    let len = input_open.len();

    let mut output_open = vec![0.0; len];
    let mut output_high = vec![0.0; len];
    let mut output_low = vec![0.0; len];
    let mut output_close = vec![0.0; len];

    py.detach(|| {
        ha::ha(
            input_open,
            input_high,
            input_low,
            input_close,
            output_open.as_mut_slice(),
            output_high.as_mut_slice(),
            output_low.as_mut_slice(),
            output_close.as_mut_slice(),
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok((
        output_open.into_pyarray(py).into(),
        output_high.into_pyarray(py).into(),
        output_low.into_pyarray(py).into(),
        output_close.into_pyarray(py).into(),
    ))
}

/// Calculates a single Heikin-Ashi candle incrementally for streaming data.
///
/// Args:
///   curr_open: Current candle's open price.
///   curr_high: Current candle's high price.
///   curr_low: Current candle's low price.
///   curr_close: Current candle's close price.
///   prev_ha_open: Previous Heikin-Ashi candle's open price.
///   prev_ha_close: Previous Heikin-Ashi candle's close price.
///
/// Returns:
///   A tuple of four values (HA_Open, HA_High, HA_Low, HA_Close).
///
/// Examples:
///   ```python
///   >>> import kand
///   >>> ha_open, ha_high, ha_low, ha_close = kand.ha_inc(11.2, 11.8, 10.8, 11.5, 10.3625, 10.875)
///   ```
#[pyfunction]
#[pyo3(name = "ha_inc", signature = (curr_open, curr_high, curr_low, curr_close, prev_ha_open, prev_ha_close))]
pub fn ha_inc_py(
    py: Python,
    curr_open: TAFloat,
    curr_high: TAFloat,
    curr_low: TAFloat,
    curr_close: TAFloat,
    prev_ha_open: TAFloat,
    prev_ha_close: TAFloat,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat)> {
    py.detach(|| ha::ha_inc(curr_open, curr_high, curr_low, curr_close, prev_ha_open, prev_ha_close))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[cfg(feature = "arrow")]
crate::kand_py_arrow_wrapper_multi!(
    ha_arrow,
    kand::ohlcv::ha::ha_arrow,
    inputs: { open, high, low, close },
    params: {},
    output_count: 4
);
