#[macro_export]
macro_rules! kand_arrow_wrapper {
    (
        $name:ident,
        $raw_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        lookback_params: { $($lb_param:ident),* }
    ) => {
        #[cfg(feature = "arrow")]
        #[allow(clippy::too_many_arguments)]
        pub fn $name(
            $($input_name: &$crate::ta::types::TAArrowArray,)+
            $($param_name: $param_type),*
        ) -> Result<$crate::ta::types::TAArrowArray, $crate::KandError> {
            use arrow::array::Array;
            use std::mem::size_of;

            // Get first input length
            let first_input = [ $( $input_name ),+ ][0];
            let len = first_input.len();

            // Validate all inputs same length and no nulls
            $(
                if $input_name.len() != len {
                    return Err($crate::KandError::LengthMismatch);
                }
                if $input_name.null_count() > 0 {
                    return Err($crate::KandError::InvalidData);
                }
            )+

            // Validation (lookback)
            let lookback = lookback($($lb_param),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            // Slices with offsets
            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            // Use pooled buffer for high performance
            let len_bytes = len * size_of::<$crate::TAFloat>();
            let (ptr, buffer) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
            let output_slice = unsafe {
                std::slice::from_raw_parts_mut(ptr as *mut $crate::TAFloat, len)
            };

            // Initialize with NAN efficiently
            output_slice.fill($crate::TAFloat::NAN);

            // Computation
            $raw_fn($($input_name,)+ $($param_name,)* output_slice);

            Ok($crate::ta::types::TAArrowArray::new(buffer.into(), None))
        }
    };
}

#[macro_export]
macro_rules! kand_arrow_wrapper_multi {
    (
        $name:ident,
        $raw_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        lookback_params: { $($lb_param:ident),* },
        outputs: { $($output_name:ident : $output_type:ty),+ },
        return_type: { $($ret_type:ty),+ }
    ) => {
        #[cfg(feature = "arrow")]
        #[allow(clippy::too_many_arguments)]
        pub fn $name(
            $($input_name: &$crate::ta::types::TAArrowArray,)+
            $($param_name: $param_type),*
        ) -> Result<($( $ret_type ),+), $crate::KandError> {
            use arrow::array::Array;
            use std::mem::size_of;

            // Get first input length
            let first_input = [ $( $input_name ),+ ][0];
            let len = first_input.len();

            // Validate all inputs same length and no nulls
            $(
                if $input_name.len() != len {
                    return Err($crate::KandError::LengthMismatch);
                }
                if $input_name.null_count() > 0 {
                    return Err($crate::KandError::InvalidData);
                }
            )+

            // Validation (lookback)
            let lookback = lookback($($lb_param),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            // Slices with offsets
            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            // Allocate outputs using pooled buffers
            $(
                let len_bytes = len * size_of::<$output_type>();
                let (ptr, $output_name) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
                let $output_name = ($output_name, unsafe {
                    std::slice::from_raw_parts_mut(ptr as *mut $output_type, len)
                });

                // Initialize with NAN if TAFloat
                if std::any::TypeId::of::<$output_type>() == std::any::TypeId::of::<$crate::TAFloat>() {
                    let slice = unsafe {
                        std::slice::from_raw_parts_mut(ptr as *mut $crate::TAFloat, len)
                    };
                    slice.fill($crate::TAFloat::NAN);
                }
            )+

            // Computation
            $raw_fn(
                $($input_name,)+
                $($param_name,)*
                $( $output_name.1 ),+
            );

            Ok(($( <$ret_type>::new($output_name.0.into(), None) ),+))
        }
    };
}

#[macro_export]
macro_rules! kand_arrow_wrapper_int {
    (
        $name:ident,
        $raw_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        lookback_params: { $($lb_param:ident),* }
    ) => {
        #[cfg(feature = "arrow")]
        #[allow(clippy::too_many_arguments)]
        pub fn $name(
            $($input_name: &$crate::ta::types::TAArrowArray,)+
            $($param_name: $param_type),*
        ) -> Result<$crate::ta::types::TAArrowIntArray, $crate::KandError> {
            use arrow::array::Array;
            use std::mem::size_of;

            // Get first input length
            let first_input = [ $( $input_name ),+ ][0];
            let len = first_input.len();

            // Validate all inputs same length and no nulls
            $(
                if $input_name.len() != len {
                    return Err($crate::KandError::LengthMismatch);
                }
                if $input_name.null_count() > 0 {
                    return Err($crate::KandError::InvalidData);
                }
            )+

            // Validation (lookback)
            let lookback = lookback($($lb_param),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            // Slices with offsets
            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            // Use pooled buffer
            let len_bytes = len * size_of::<$crate::TAInt>();
            let (ptr, buffer) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
            let output_slice = unsafe {
                std::slice::from_raw_parts_mut(ptr as *mut $crate::TAInt, len)
            };

            // Computation
            $raw_fn($($input_name,)+ $($param_name,)* output_slice);

            Ok($crate::ta::types::TAArrowIntArray::new(buffer.into(), None))
        }
    };
}
