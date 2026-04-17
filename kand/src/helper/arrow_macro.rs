#[macro_export]
macro_rules! kand_arrow_wrapper {
    (
        $name:ident,
        $raw_fn:path,
        inputs: { $($input_name:ident),+ },
        params: { $($param_name:ident : $param_type:ty),* }
    ) => {
        #[cfg(feature = "arrow")]
        pub fn $name(
            $($input_name: &$crate::ta::types::TAArrowArray,)+
            $($param_name: $param_type),*
        ) -> Result<$crate::ta::types::TAArrowArray, $crate::KandError> {
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
            let lookback = $crate::ta::ohlcv::$name::lookback($($param_name),*)?;
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

            // Computation
            $raw_fn($($input_name,)+ $($param_name,)* output_slice);

            // Fill NaNs
            #[cfg(feature = "allow-nan")]
            {
                for value in output_slice.iter_mut().take(lookback) {
                    *value = $crate::TAFloat::NAN;
                }
            }

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
        outputs: { $($output_name:ident),+ }
    ) => {
        #[cfg(feature = "arrow")]
        pub fn $name(
            $($input_name: &$crate::ta::types::TAArrowArray,)+
            $($param_name: $param_type),*
        ) -> Result<($( $crate::ta::types::TAArrowArray ),+), $crate::KandError> {
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
            let lookback = $crate::ta::ohlcv::$name::lookback($($param_name),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            // Slices with offsets
            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            // Aligned allocation for each output
            $(
                let mut $output_name = MutableBuffer::new(len * size_of::<$crate::TAFloat>());
                $output_name.resize(len * size_of::<$crate::TAFloat>(), 0);
            )+

            // Computation
            $raw_fn(
                $($input_name,)+ 
                $($param_name,)* 
                $( $output_name.typed_data_mut::<$crate::TAFloat>() ),+
            );

            // Fill NaNs
            #[cfg(feature = "allow-nan")]
            {
                $(
                    for value in $output_name.typed_data_mut::<$crate::TAFloat>().iter_mut().take(lookback) {
                        *value = $crate::TAFloat::NAN;
                    }
                )+
            }

            Ok(($( $crate::ta::types::TAArrowArray::new($output_name.into(), None) ),+))
        }
    };
}
