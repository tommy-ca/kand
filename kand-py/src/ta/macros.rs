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
        #[pyo3(name = stringify!($name), signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<pyo3_arrow::PyArray> {
            use std::sync::Arc;
            use kand::ta::types::TAArrowArray;

            let first_input = [ $( &$input_name ),+ ][0];
            
            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+

            let result = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok(pyo3_arrow::PyArray::new(Arc::new(result), first_input.field().clone()))
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
        #[pyo3(name = stringify!($name), signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use kand::ta::types::TAArrowArray;

            let first_input = [ $( &$input_name ),+ ][0];
            
            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+

            let (r1, r2) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), first_input.field().clone())
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
        #[pyo3(name = stringify!($name), signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use kand::ta::types::TAArrowArray;

            let first_input = [ $( &$input_name ),+ ][0];
            
            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+

            let (r1, r2, r3) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), first_input.field().clone())
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
        #[pyo3(name = stringify!($name), signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use kand::ta::types::TAArrowArray;

            let first_input = [ $( &$input_name ),+ ][0];
            
            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+

            let (r1, r2, r3, r4) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), first_input.field().clone())
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
        #[pyo3(name = stringify!($name), signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use kand::ta::types::TAArrowArray;

            let first_input = [ $( &$input_name ),+ ][0];
            
            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+

            let (r1, r2, r3, r4, r5) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r5), first_input.field().clone())
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
        #[pyo3(name = stringify!($name), signature = ($($input_name,)+ $($param_name),*))]
        pub fn $name(
            py: pyo3::prelude::Python,
            $($input_name: pyo3_arrow::PyArray,)+
            $($param_name: $param_type),*
        ) -> pyo3::prelude::PyResult<(pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray, pyo3_arrow::PyArray)> {
            use std::sync::Arc;
            use kand::ta::types::TAArrowArray;

            let first_input = [ $( &$input_name ),+ ][0];
            
            $(
                let $input_name = $input_name.as_ref()
                    .as_any()
                    .downcast_ref::<TAArrowArray>()
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err(format!("Expected compatible Arrow floating-point array for {}", stringify!($input_name))))?;
            )+

            let (r1, r2, r3, r4, r5, r6, r7) = py.allow_threads(|| $arrow_fn($($input_name,)+ $($param_name),*))
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            Ok((
                pyo3_arrow::PyArray::new(Arc::new(r1), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r2), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r3), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r4), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r5), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r6), first_input.field().clone()),
                pyo3_arrow::PyArray::new(Arc::new(r7), first_input.field().clone())
            ))
        }
    };
}
