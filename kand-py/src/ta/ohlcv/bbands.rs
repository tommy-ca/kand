use kand::ta::types::MAType;
use kand::{TAFloat, ohlcv::bbands};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (price, period, dev_up, dev_down, ma_type))]
pub fn bbands_py(
    py: Python,
    price: PyReadonlyArray1<TAFloat>,
    period: usize,
    dev_up: TAFloat,
    dev_down: TAFloat,
    ma_type: u32,
) -> PyResult<(
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
    Py<PyArray1<TAFloat>>,
)> {
    let input_price = price.as_slice()?;
    let len = input_price.len();
    let ma_type = MAType::try_from(ma_type as i64)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;

    let mut output_upper = vec![0.0; len];
    let mut output_middle = vec![0.0; len];
    let mut output_lower = vec![0.0; len];

    py.detach(|| {
        let _ = bbands::bbands(
            input_price,
            period,
            dev_up,
            dev_down,
            ma_type,
            &mut output_upper,
            &mut output_middle,
            &mut output_lower,
        );
        Ok::<(), PyErr>(())
    })?;

    Ok((
        output_upper.into_pyarray(py).into(),
        output_middle.into_pyarray(py).into(),
        output_lower.into_pyarray(py).into(),
    ))
}

#[pyfunction]
#[pyo3(signature = (price, prev_sma, prev_sum, prev_sum_sq, old_price, period, dev_up, dev_down, ma_type))]
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
    ma_type: u32,
) -> PyResult<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat)> {
    let ma_type = MAType::try_from(ma_type as i64)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;

    py.detach(|| {
        bbands::bbands_inc(
            price,
            prev_sma,
            prev_sum,
            prev_sum_sq,
            old_price,
            period,
            dev_up,
            dev_down,
            ma_type,
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    })
}

// Arrow wrapper
#[cfg(feature = "arrow")]
#[pyo3::prelude::pyfunction]
#[pyo3(signature = (price, period, multiplier_up, multiplier_down, ma_type))]
pub fn bbands_arrow(
    py: pyo3::prelude::Python,
    price: pyo3_arrow::PyArray,
    period: usize,
    multiplier_up: TAFloat,
    multiplier_down: TAFloat,
    ma_type: i64,
) -> pyo3::prelude::PyResult<(
    pyo3_arrow::PyArray,
    pyo3_arrow::PyArray,
    pyo3_arrow::PyArray,
)> {
    use arrow::array::Array;
    use kand::ta::types::{MAType, TAArrowArray};
    use std::sync::Arc;
    let ma_type = MAType::try_from(ma_type)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid MAType"))?;

    let price_array = price
        .as_ref()
        .as_any()
        .downcast_ref::<TAArrowArray>()
        .ok_or_else(|| {
            pyo3::exceptions::PyTypeError::new_err(
                "Expected compatible Arrow floating-point array for price",
            )
        })?;

    let (r1, r2, r3) = py
        .detach(|| {
            kand::ta::ohlcv::bbands::bbands_arrow(
                price_array,
                period,
                multiplier_up,
                multiplier_down,
                ma_type,
            )
        })
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let field = Arc::new(arrow::datatypes::Field::new(
        "",
        price_array.data_type().clone(),
        true,
    ));
    Ok((
        pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
        pyo3_arrow::PyArray::new(Arc::new(r2), field.clone()),
        pyo3_arrow::PyArray::new(Arc::new(r3), field.clone()),
    ))
}
