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
            use arrow::buffer::MutableBuffer;
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

            // Aligned allocation
            let mut buffer = MutableBuffer::new(len * size_of::<$crate::TAFloat>());
            buffer.resize(len * size_of::<$crate::TAFloat>(), 0);
            let output_slice = buffer.typed_data_mut::<$crate::TAFloat>();
            
            // Initialize with NAN
            for value in output_slice.iter_mut() {
                *value = $crate::TAFloat::NAN;
            }

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
            use arrow::buffer::MutableBuffer;
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

            // Aligned allocation for each output
            $(
                let mut $output_name = MutableBuffer::new(len * size_of::<$output_type>());
                $output_name.resize(len * size_of::<$output_type>(), 0);
                
                // Initialize with NAN if TAFloat
                if std::any::TypeId::of::<$output_type>() == std::any::TypeId::of::<$crate::TAFloat>() {
                    let slice = $output_name.typed_data_mut::<$crate::TAFloat>();
                    for value in slice.iter_mut() {
                        *value = $crate::TAFloat::NAN;
                    }
                }
            )+

            // Computation
            $raw_fn(
                $($input_name,)+ 
                $($param_name,)* 
                $( $output_name.typed_data_mut::<$output_type>() ),+
            );

            Ok(($( <$ret_type>::new($output_name.into(), None) ),+))
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
            use arrow::buffer::MutableBuffer;
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

            // Aligned allocation
            let mut buffer = MutableBuffer::new(len * size_of::<$crate::TAInt>());
            buffer.resize(len * size_of::<$crate::TAInt>(), 0);
            let output_slice = buffer.typed_data_mut::<$crate::TAInt>();

            // Computation
            $raw_fn($($input_name,)+ $($param_name,)* output_slice);

            Ok($crate::ta::types::TAArrowIntArray::new(buffer.into(), None))
        }
    };
}
