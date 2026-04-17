use kand::ohlcv::cdl_dragonfly_doji;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Detects Dragonfly Doji candlestick patterns in price data.
...
///   >>> signal = kand.cdl_dragonfly_doji_inc(10.0, 10.1, 8.5, 10.05, 5.0, 10.0)
///   ```
#[pyfunction]
#[pyo3(name = "cdl_dragonfly_doji_inc", signature = (open, high, low, close, body_percent, shadow_percent))]
pub fn cdl_dragonfly_doji_inc_py(
    py: Python,
    open: TAFloat,
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    body_percent: TAFloat,
    shadow_percent: TAFloat,
) -> PyResult<TAInt> {
    py.allow_threads(|| {
        cdl_dragonfly_doji::cdl_dragonfly_doji_inc(
            open,
            high,
            low,
            close,
            body_percent,
            shadow_percent,
        )
    })
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_int!(
    cdl_dragonfly_doji_arrow,
    kand::ta::ohlcv::cdl_dragonfly_doji::cdl_dragonfly_doji_arrow,
    inputs: { open, high, low, close },
    params: { body_percent: TAFloat, shadow_percent: TAFloat }
);
