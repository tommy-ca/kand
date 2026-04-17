use kand::ohlcv::tema;
use pyo3::prelude::*;

/// Computes the Triple Exponential Moving Average (TEMA) over a NumPy array.
...
///   >>> tema, ema1, ema2, ema3 = kand.tema_inc(10.0, 9.0, 8.0, 7.0, 3)
///   ```
#[pyfunction]
#[pyo3(name = "tema_inc", signature = (price, prev_ema1, prev_ema2, prev_ema3, period))]
pub fn tema_inc_py(
    py: Python,
    price: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    prev_ema3: TAFloat,
    period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat)> {
    py.allow_threads(|| tema::tema_inc(price, prev_ema1, prev_ema2, prev_ema3, period))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    tema_arrow,
    kand::ta::ohlcv::tema::tema_arrow,
    inputs: { data },
    params: { period: usize },
    output_count: 4
);
