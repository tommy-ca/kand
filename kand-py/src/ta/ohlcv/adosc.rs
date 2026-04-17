use kand::ohlcv::adosc;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Calculates the Accumulation/Distribution Oscillator (A/D Oscillator or ADOSC) for the entire price series.
...
///   >>> adosc, ad, ad_fast_ema, ad_slow_ema = kand.adosc_inc(
///   ...     10.5, 9.5, 10.0, 150.0, 100.0, 95.0, 90.0, 3, 10)
///   ```
#[pyfunction]
#[pyo3(name = "adosc_inc", signature = (high, low, close, volume, prev_ad, prev_ad_fast_ema, prev_ad_slow_ema, fast_period, slow_period))]
pub fn adosc_inc_py(
    high: TAFloat,
    low: TAFloat,
    close: TAFloat,
    volume: TAFloat,
    prev_ad: TAFloat,
    prev_ad_fast_ema: TAFloat,
    prev_ad_slow_ema: TAFloat,
    fast_period: usize,
    slow_period: usize,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat)> {
    adosc::adosc_inc(
        high,
        low,
        close,
        volume,
        prev_ad,
        prev_ad_fast_ema,
        prev_ad_slow_ema,
        fast_period,
        slow_period,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    adosc_arrow,
    kand::ta::ohlcv::adosc::adosc_arrow,
    inputs: { high, low, close, volume },
    params: { fast_period: usize, slow_period: usize },
    output_count: 4
);
