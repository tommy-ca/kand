use kand::ohlcv::trima;
use pyo3::prelude::*;

/// Computes the Triangular Moving Average (TRIMA) over a NumPy array.
...
///   >>> trima, sma1 = kand.trima_inc(35.5, 35.2, 36.0, 35.0, 35.1, 5)
///   ```
#[pyfunction]
#[pyo3(name = "trima_inc", signature = (prev_sma1, prev_sma2, new_price, old_price, old_sma1, period))]
pub fn trima_inc_py(
    py: Python,
    prev_sma1: TAFloat,
    prev_sma2: TAFloat,
    new_price: TAFloat,
    old_price: TAFloat,
    old_sma1: TAFloat,
    period: usize,
) -> PyResult<(TAFloat, TAFloat)> {
    py.allow_threads(|| trima::trima_inc(prev_sma1, prev_sma2, new_price, old_price, old_sma1, period))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    trima_arrow,
    kand::ta::ohlcv::trima::trima_arrow,
    inputs: { data },
    params: { period: usize },
    output_count: 2
);
