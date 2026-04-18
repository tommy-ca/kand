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

            // Use pooled buffer
            let len_bytes = len * size_of::<$crate::TAInt>();
            let (ptr, buffer) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
            let output_slice = unsafe {
                std::slice::from_raw_parts_mut(ptr as *mut $crate::TAInt, len)
            };

            // Computation
            $raw_fn($($input_name,)+ $($param_name,)* output_slice);

            Ok($crate::ta::types::TAArrowArray::new(buffer.into(), None))
        }
    };
}

#[macro_export]
macro_rules! kand_indicator {
    (
        $name:ident,
        inputs: { $input_name:ident : $input_type:ty },
        params: { $($param_name:ident : $param_type:ty),* },
        state: { $($state_field:ident : $state_type:ty),* },
        init: |$init_params:ident| $init_block:block,
        next: |$next_state:ident, $next_input:ident| $next_block:block
    ) => {
        $crate::paste::paste! {
            /// Stateful implementation generated by macro.
            #[derive(Clone)]
            pub struct [<Stateful $name:camel>] {
                $($param_name : $param_type,)*
                $($state_field : $state_type,)*
                __kand_count: usize,
            }

            impl [<Stateful $name:camel>] {
                /// Creates a new stateful instance.
                pub fn new($($param_name : $param_type),*) -> Result<Self, $crate::KandError> {
                    let $init_params = ($($param_name),*);
                    let state = $init_block;
                    Ok(Self {
                        $($param_name,)*
                        $($state_field : state.$state_field,)*
                        __kand_count: 0,
                    })
                }
            }

            impl $crate::ta::traits::Indicator for [<Stateful $name:camel>] {
                type Input = $input_type;
                type Output = $crate::TAFloat;

                fn next(&mut self, $next_input: Self::Input) -> Result<Self::Output, $crate::KandError> {
                    self.__kand_count += 1;
                    let $next_state = self;
                    $next_block
                }

                #[cfg(feature = "arrow")]
                fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    use arrow::datatypes::{DataType, Field, Schema};
                    use std::sync::Arc;

                    // Automated schema generation for scalars
                    let mut fields = vec![
                        Field::new("__kand_count", DataType::UInt64, false),
                    ];
                    $(
                        // This is a simplified prototype. Real implementation would need to match types.
                        fields.push(Field::new(stringify!($param_name), DataType::Float64, false));
                    )*

                    // ... persistence logic ...
                    Err($crate::KandError::InvalidData)
                }

                #[cfg(feature = "arrow")]
                fn restore_from_record_batch(&mut self, _batch: &arrow::record_batch::RecordBatch) -> Result<(), $crate::KandError> {
                    Err($crate::KandError::InvalidData)
                }
            }
        }
    };
}
