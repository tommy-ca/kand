use arrow::array::Array;
use kand::{TAFloat, ohlcv::adosc, ta::types::MAType};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculate Accumulation/Distribution Oscillator (A/D Oscillator or ADOSC)
///
/// The A/D Oscillator is a momentum indicator that measures the difference between a fast and slow EMA of the
/// Accumulation/Distribution Line. It helps identify trend strength and potential reversals.
///
/// Args:
///   high: High prices as a 1-D NumPy array of type `TAFloat`.
///   low: Low prices as a 1-D NumPy array of type `TAFloat`.
///   close: Close prices as a 1-D NumPy array of type `TAFloat`.
///   volume: Volume as a 1-D NumPy array of type `TAFloat`.
///   fast_period: Fast period for A/D Oscillator calculation.
///   slow_period: Slow period for A/D Oscillator calculation.
///
/// Returns:
///   A 1-D NumPy array containing ADOSC values.
///
/// Examples:
///   ```python
///   >>> import numpy as np
///   >>> import kand
///   >>> high = np.array([10.0, 11.0, 12.0, 11.5, 10.5])
///   >>> low = np.array([8.0, 9.0, 10.0, 9.5, 8.5])
///   >>> close = np.array([9.0, 10.0, 11.0, 10.0, 9.0])
///   >>> volume = np.array([100.0, 150.0, 200.0, 150.0, 100.0])
///   >>> adosc = kand.adosc(high, low, close, volume, 3, 5, 0)
///   ```
#[pyfunction]
#[pyo3(name = "adosc", signature = (high, low, close, volume, fast_period, slow_period, ma_type))]
pub fn adosc_py(
    py: Python,
    high: PyReadonlyArray1<TAFloat>,
    low: PyReadonlyArray1<TAFloat>,
    close: PyReadonlyArray1<TAFloat>,
    volume: PyReadonlyArray1<TAFloat>,
    fast_period: usize,
    slow_period: usize,
    ma_type: u32,
) -> PyResult<Py<PyArray1<TAFloat>>> {
    // Convert input NumPy arrays to Rust slices
    let input_high = high.as_slice()?;
    let input_low = low.as_slice()?;
    let input_close = close.as_slice()?;
    let input_volume = volume.as_slice()?;
    let len = input_high.len();

    // Create output arrays
    let mut output_adosc = vec![0.0; len];

    let ma_type = MAType::try_from(ma_type as i64)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;

    // Perform ADOSC calculation while releasing the GIL
    py.detach(|| {
        let _ = adosc::adosc(
            input_high,
            input_low,
            input_close,
            input_volume,
            fast_period,
            slow_period,
            ma_type,
            &mut output_adosc,
        );
        Ok::<(), PyErr>(())
    })?;

    // Convert output arrays to Python objects
    Ok(output_adosc.into_pyarray(py).into())
}

/// Calculate latest A/D Oscillator value incrementally
///
/// Provides optimized calculation of the latest ADOSC value when new data arrives,
/// without recalculating the entire series.
///
/// Args:
///     high: Latest high price.
///     low: Latest low price.
///     close: Latest closing price.
///     volume: Latest volume.
///     prev_ad: Previous A/D value.
///     prev_fast_ema: Previous fast EMA value.
///     prev_slow_ema: Previous slow EMA value.
///     fast_period: Fast EMA period.
///     slow_period: Slow EMA period.
///     ma_type: Moving average type.
///
/// Returns:
///     A tuple containing (ADOSC, AD, Fast EMA, Slow EMA) values.
///
/// Examples:
///     ```python
///     >>> import kand
///     >>> adosc, ad, fast_ema, slow_ema = kand.adosc_inc(
///     ...     10.5,  # high
///     ...     9.5,   # low
///     ...     10.0,  # close
///     ...     150.0, # volume
///     ...     100.0, # prev_ad
///     ...     95.0,  # prev_fast_ema
///     ...     90.0,  # prev_slow_ema
///     ...     3,     # fast_period
///     ...     10,    # slow_period
///     ...     0      # ma_type (SMA/EMA)
///     ... )
///     ```
#[pyfunction]
#[pyo3(name = "adosc_inc", signature = (high, low, close, volume, prev_ad, prev_fast_ema, prev_slow_ema, fast_period, slow_period, ma_type))]
pub fn adosc_inc_py(
    py: Python,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    volume: TAFloat,
    prev_ad: TAFloat,
    prev_fast_ema: TAFloat,
    prev_slow_ema: TAFloat,
    fast_period: usize,
    slow_period: usize,
    ma_type: u32,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat)> {
    let ma_type = MAType::try_from(ma_type as i64)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;

    // Perform the incremental ADOSC calculation while releasing the GIL
    py.detach(|| {
        adosc::adosc_inc(
            high,
            low,
            close,
            volume,
            prev_ad,
            prev_fast_ema,
            prev_slow_ema,
            fast_period,
            slow_period,
            ma_type,
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    })
}

// Arrow wrapper
#[cfg(feature = "arrow")]
#[pyo3::prelude::pyfunction]
#[pyo3(signature = (high, low, close, volume, fast_period, slow_period, ma_type))]
pub fn adosc_arrow(
    py: pyo3::prelude::Python,
    high: pyo3_arrow::PyArray,
    low: pyo3_arrow::PyArray,
    close: pyo3_arrow::PyArray,
    volume: pyo3_arrow::PyArray,
    fast_period: usize,
    slow_period: usize,
    ma_type: i64,
) -> pyo3::prelude::PyResult<pyo3_arrow::PyArray> {
    use kand::ta::types::{MAType, TAArrowArray};
    use std::sync::Arc;
    let ma_type = MAType::try_from(ma_type)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;

    let high_array = high
        .as_ref()
        .as_any()
        .downcast_ref::<TAArrowArray>()
        .ok_or_else(|| {
            pyo3::exceptions::PyTypeError::new_err(
                "Expected compatible Arrow floating-point array for high",
            )
        })?;
    let low = low
        .as_ref()
        .as_any()
        .downcast_ref::<TAArrowArray>()
        .ok_or_else(|| {
            pyo3::exceptions::PyTypeError::new_err(
                "Expected compatible Arrow floating-point array for low",
            )
        })?;
    let close = close
        .as_ref()
        .as_any()
        .downcast_ref::<TAArrowArray>()
        .ok_or_else(|| {
            pyo3::exceptions::PyTypeError::new_err(
                "Expected compatible Arrow floating-point array for close",
            )
        })?;
    let volume = volume
        .as_ref()
        .as_any()
        .downcast_ref::<TAArrowArray>()
        .ok_or_else(|| {
            pyo3::exceptions::PyTypeError::new_err(
                "Expected compatible Arrow floating-point array for volume",
            )
        })?;

    let result = py
        .detach(|| {
            kand::ta::ohlcv::adosc::adosc_arrow(
                high_array,
                low,
                close,
                volume,
                fast_period,
                slow_period,
                ma_type,
            )
        })
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let field = Arc::new(arrow::datatypes::Field::new(
        "",
        high_array.data_type().clone(),
        true,
    ));
    Ok(pyo3_arrow::PyArray::new(Arc::new(result), field))
}
