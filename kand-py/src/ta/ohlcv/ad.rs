use kand::ohlcv::ad;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates the Accumulation/Distribution (A/D) indicator for the entire price series.
...
///   >>> output_ad = kand.ad_inc(15.0, 11.0, 13.0, 200.0, 25.0)
///   ```
#[pyfunction]
#[pyo3(name = "ad_inc", signature = (high, low, close, volume, prev_ad))]
pub fn ad_inc_py(
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    volume: TAFloat,
    prev_ad: TAFloat,
) -> PyResult<TAFloat> {
    ad::ad_inc(high, low, close, volume, prev_ad)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper!(
    ad_arrow,
    kand::ta::ohlcv::ad::ad_arrow,
    inputs: { high, low, close, volume },
    params: {}
);
