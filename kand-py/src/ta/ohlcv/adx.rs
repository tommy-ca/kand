use kand::ohlcv::adx;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculate Average Directional Index (ADX) for a NumPy array
...
///   >>> adx, plus_dm, minus_dm, tr = kand.adx_inc(
///   ...     24.20,  # current high
///   ...     23.85,  # current low
///   ...     24.07,  # previous high
///   ...     23.72,  # previous low
///   ...     23.95,  # previous close
///   ...     25.0,   # previous ADX
///   ...     0.5,    # previous smoothed +DM
///   ...     0.3,    # previous smoothed -DM
///   ...     1.2,    # previous smoothed TR
///   ...     14      # period
///   ... )
///   ```
pub fn adx_inc_py(
    py: Python,
    high: TAFloat,
    low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    prev_close: TAFloat,
    prev_adx: TAFloat,
    prev_smoothed_plus_dm: TAFloat,
    prev_smoothed_minus_dm: TAFloat,
    prev_smoothed_tr: TAFloat,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat)> {
    // Perform incremental ADX calculation while releasing the GIL
    py.allow_threads(|| {
        adx::adx_inc(
            high,
            low,
            prev_high,
            prev_low,
            prev_close,
            prev_adx,
            prev_smoothed_plus_dm,
            prev_smoothed_minus_dm,
            prev_smoothed_tr,
            period,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    adx_arrow,
    kand::ta::ohlcv::adx::adx_arrow,
    inputs: { high, low, close },
    params: { period: usize },
    output_count: 4
);
