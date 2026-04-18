use kand::{ta::types::MAType, ohlcv::bbands, TAFloat};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculate Bollinger Bands for a NumPy array.
///
/// Bollinger Bands consist of:
/// - A middle band (N-period simple moving average)
/// - A lower band (K standard deviations below middle band)
/// - An upper band (K standard deviations above middle band)
///
/// Args:
///   price: Input price values as a 1-D NumPy array of type `TAFloat`.
///   period: The time period for calculations (must be >= 2).
///   dev_up: Number of standard deviations for upper band.
///   dev_down: Number of standard deviations for lower band.
///
/// Returns:
///   A tuple of 3 1-D NumPy arrays containing:
///   - Upper band values
///   - Middle band values
///   - Lower band values
///   The first (period-1) elements of each array contain NaN values.
///
/// Examples:
///   ```python
///   >>> import numpy as np
///   >>> import kand
///   >>> price = np.array([10.0, 11.0, 12.0, 13.0, 14.0])
///   >>> upper, middle, lower = kand.bbands(price, 3, 2.0, 2.0)
///   ```
#[pyfunction]
#[pyo3(name = "bbands", signature = (price, period, dev_up, dev_down, ma_type))]
pub fn bbands_py(
    py: Python,
    price: PyReadonlyArray1<TAFloat>,
    period: usize,
    dev_up: TAFloat,
    dev_down: TAFloat,
    ma_type: u32,
) -> PyResult<(
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
)> {
    let price_slice = price.as_slice()?;
    let len = price_slice.len();

    let mut output_upper = vec![0.0; len];
    let mut output_middle = vec![0.0; len];
    let mut output_lower = vec![0.0; len];

    py.allow_threads(|| {
        let ma_type = MAType::try_from(ma_type as i64)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;
        bbands::bbands(
            price_slice,
            period,
            dev_up,
            dev_down,
            ma_type,
            output_upper.as_mut_slice(),
            output_middle.as_mut_slice(),
            output_lower.as_mut_slice(),
        );
        Ok(())
    })?;

    Ok((
        output_upper.into_pyarray(py).into(),
        output_middle.into_pyarray(py).into(),
        output_lower.into_pyarray(py).into(),
    ))
}

#[pyfunction]
#[pyo3(name = "bbands_inc", signature = (
    price,
    prev_sma,
    prev_sum,
    prev_sum_sq,
    old_price,
    period,
    dev_up,
    dev_down,
    ma_type
))]
pub fn bbands_inc_py(
    py: Python,
    price: TAFloat,
    prev_sma: TAFloat,
    prev_sum: TAFloat,
    prev_sum_sq: TAFloat,
    old_price: TAFloat,
    period: usize,
    dev_up: TAFloat,
    dev_down: TAFloat,
    ma_type: u32,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat)> {
    py.allow_threads(|| {
        let ma_type = MAType::try_from(ma_type as i64).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err("Invalid MAType")
        py.allow_threads(|| {
            let ma_type = MAType::try_from(ma_type as i64)
                .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;
            bbands::bbands_inc(
                price,
                prev_sma,
                prev_sum,
                prev_sum_sq,
                old_price,
                period,
                dev_up,
                dev_down,
                ma_type,
            )
        })
        }

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    bbands_arrow,
    kand::ta::ohlcv::bbands::bbands_arrow,
    inputs: { price },
    params: {
        period: usize,
        dev_up: TAFloat,
        dev_down: TAFloat,
        ma_type: u32
    },
    output_count: 3
);
