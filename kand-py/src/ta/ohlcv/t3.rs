use kand::ohlcv::t3;
use pyo3::prelude::*;

/// Computes the T3 (Triple Exponential Moving Average) over a NumPy array.
...
///   >>> t3, ema1, ema2, ema3, ema4, ema5, ema6 = kand.t3_inc(100.0, 95.0, 94.0, 93.0, 92.0, 91.0, 90.0, 5, 0.7)
///   ```
#[pyfunction]
#[pyo3(name = "t3_inc", signature = (price, prev_ema1, prev_ema2, prev_ema3, prev_ema4, prev_ema5, prev_ema6, period, vfactor))]
pub fn t3_inc_py(
    py: Python,
    price: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    prev_ema3: TAFloat,
    prev_ema4: TAFloat,
    prev_ema5: TAFloat,
    prev_ema6: TAFloat,
    period: usize,
    vfactor: TAFloat,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat)> {
    py.allow_threads(|| t3::t3_inc(price, prev_ema1, prev_ema2, prev_ema3, prev_ema4, prev_ema5, prev_ema6, period, vfactor))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    t3_arrow,
    kand::ta::ohlcv::t3::t3_arrow,
    inputs: { data },
    params: { period: usize, vfactor: TAFloat },
    output_count: 7
);
