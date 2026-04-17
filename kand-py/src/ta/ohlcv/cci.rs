use kand::ohlcv::cci;
use pyo3::prelude::*;

/// Calculates the Commodity Channel Index (CCI) for a NumPy array.
...
///   >>> next_cci = kand.cci_inc(
///   ...     100.0,   # prev_sma_tp
///   ...     105.0,   # new_high
///   ...     95.0,    # new_low
///   ...     100.0,   # new_close
///   ...     102.0,   # old_high
///   ...     98.0,    # old_low
///   ...     100.0,   # old_close
///   ...     14,      # period
///   ...     tp_buffer # circular buffer
///   ... )
///   ```
#[pyfunction]
#[pyo3(name = "cci_inc", signature = (
    prev_sma_tp,
    input_new_high,
    input_new_low,
    input_new_close,
    input_old_high,
    input_old_low,
    input_old_close,
    opt_period,
    tp_buffer
))]
pub fn cci_inc_py(
    py: Python,
    prev_sma_tp: TAFloat,
    input_new_high: TAFloat,
    input_new_low: TAFloat,
    input_new_close: TAFloat,
    input_old_high: TAFloat,
    input_old_low: TAFloat,
    input_old_close: TAFloat,
    opt_period: usize,
    tp_buffer: &mut Vec<TAFloat>,
) -> PyResult<TAFloat> {
    py.allow_threads(|| {
        cci::cci_inc(
            prev_sma_tp,
            input_new_high,
            input_new_low,
            input_new_close,
            input_old_high,
            input_old_low,
            input_old_close,
            opt_period,
            tp_buffer,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    cci_arrow,
    kand::ta::ohlcv::cci::cci_arrow,
    inputs: { high, low, close },
    params: { period: usize },
    output_count: 4
);
