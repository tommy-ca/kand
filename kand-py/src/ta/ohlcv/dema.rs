use kand::ohlcv::dema;
use pyo3::prelude::*;

/// Computes the Double Exponential Moving Average (DEMA) over a NumPy array.
...
///   >>> dema, ema1, ema2 = kand.dema_inc(10.0, 9.5, 9.0, 3)
///   ```
#[pyfunction]
#[pyo3(name = "dema_inc", signature = (price, prev_ema1, prev_ema2, period))]
pub fn dema_inc_py(
    py: Python,
    price: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat)> {
    py.allow_threads(|| dema::dema_inc(price, prev_ema1, prev_ema2, period))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    dema_arrow,
    kand::ta::ohlcv::dema::dema_arrow,
    inputs: { data },
    params: { period: usize },
    output_count: 3
);
