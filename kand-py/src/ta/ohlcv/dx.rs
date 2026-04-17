use kand::ohlcv::dx;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Directional Movement Index (DX) over NumPy arrays.
...
///     >>> dx, plus_dm, minus_dm, tr = kand.dx_inc(
///     ...     high, low, prev_high, prev_low, prev_close,
///     ...     prev_smoothed_plus_dm, prev_smoothed_minus_dm,
///     ...     prev_smoothed_tr, period)
pub fn dx_inc_py(
    py: Python,
    input_high: TAFloat,
    input_low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    prev_close: TAFloat,
    prev_smoothed_plus_dm: TAFloat,
    prev_smoothed_minus_dm: TAFloat,
    prev_smoothed_tr: TAFloat,
    opt_period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat)> {
    // Perform the incremental DX calculation while releasing the GIL
    py.allow_threads(|| {
        dx::dx_inc(
            input_high,
            input_low,
            prev_high,
            prev_low,
            prev_close,
            prev_smoothed_plus_dm,
            prev_smoothed_minus_dm,
            prev_smoothed_tr,
            opt_period,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    dx_arrow,
    kand::ta::ohlcv::dx::dx_arrow,
    inputs: { high, low, close },
    params: { period: usize },
    output_count: 4
);
