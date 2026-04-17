use kand::ohlcv::sma;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Computes the Simple Moving Average (SMA) over a NumPy array.
...
/// )
/// .unwrap();
/// ```
pub fn sma_inc_py(
    prev_sma: TAFloat,
    new_price: TAFloat,
    old_price: TAFloat,
    period: usize,
) -> PyResult<TAFloat> {
    sma::sma_inc(prev_sma, new_price, old_price, period)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    sma_arrow,
    kand::ta::ohlcv::sma::sma_arrow,
    inputs: { data },
    params: { period: usize }
);
