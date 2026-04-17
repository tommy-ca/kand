use kand::ohlcv::bbands;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculate Bollinger Bands for a NumPy array.
...
///   >>> upper, middle, lower, sma, var, sum, sum_sq = kand.bbands_inc(
///   ...     10.0,   # price
///   ...     9.5,    # prev_sma
///   ...     28.5,   # prev_sum
///   ...     272.25, # prev_sum_sq
///   ...     9.0,    # old_price
///   ...     3,      # period
///   ...     2.0,    # dev_up
///   ...     2.0     # dev_down
///   ... )
///   ```
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
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat)> {
    py.allow_threads(|| {
        bbands::bbands_inc(
            price,
            prev_sma,
            prev_sum,
            prev_sum_sq,
            old_price,
            period,
            dev_up,
            dev_down,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    bbands_arrow,
    kand::ta::ohlcv::bbands::bbands_arrow,
    inputs: { price },
    params: {
        period: usize,
        dev_up: TAFloat,
        dev_down: TAFloat
    },
    output_count: 7
);
