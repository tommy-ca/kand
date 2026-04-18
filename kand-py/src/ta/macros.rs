#[macro_export]
macro_rules! kand_py_arrow_wrapper {
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* }
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<pyo3_arrow::PyArray> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let result = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok(pyo3_arrow::PyArray::new(Arc::new(result), field))
        }
    };
}

#[macro_export]
macro_rules! kand_py_arrow_wrapper_int {
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* }
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<pyo3_arrow::PyArray> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let result = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok(pyo3_arrow::PyArray::new(Arc::new(result), field))
        }
    };
}

#[macro_export]
macro_rules! kand_py_arrow_wrapper_multi {
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 2
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone())
            ))
        }
    };
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 3
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2, r3) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), field.clone())
            ))
        }
    };
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 4
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2, r3, r4) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), field.clone())
            ))
        }
    };
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 5
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2, r3, r4, r5) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r5), field.clone())
            ))
        }
    };
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 6
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2, r3, r4, r5, r6) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r5), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r6), field.clone())
            ))
        }
    };
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 2,
        output_types: { int, float }
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone())
            ))
        }
    };
    (
        $name:ident,
        $arrow_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        output_count: 7
    ) => {
        #[cfg(feature = "arrow")]
        #[pyo3::prelude::pyfunction]
        #[pyo3(signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use arrow::array::Array;
            use kand::ta::types::TAArrowArray;

            let inputs = [$($input_name.as_ref()),+];
            let first_input = inputs[0];

            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+
            let first_input_field = first_input.data_type().clone();
            let field = Arc::new(arrow::datatypes::Field::new("", first_input_field, true));

            let (r1, r2, r3, r4, r5, r6, r7) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r5), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r6), field.clone()),
                pyo3_arrow::PyArray::new(Arc::new(r7), field.clone())
            ))
        }
    };
}
