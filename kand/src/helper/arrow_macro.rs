#[macro_export]
macro_rules! id_type {
    ($name:ident, $at:ty) => { $at };
}

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

            let first_input = [ $( $input_name ),+ ][0];
            let len = first_input.len();

            $(
                if $input_name.len() != len {
                    return Err($crate::KandError::LengthMismatch);
                }
                if $input_name.null_count() > 0 {
                    return Err($crate::KandError::InvalidData);
                }
            )+

            let lookback = lookback($($lb_param),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            let len_bytes = len * size_of::<$crate::TAFloat>();
            let (ptr, buffer) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
            let output_slice = unsafe {
                std::slice::from_raw_parts_mut(ptr as *mut $crate::TAFloat, len)
            };

            output_slice.fill($crate::TAFloat::NAN);
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

            let first_input = [ $( $input_name ),+ ][0];
            let len = first_input.len();

            $(
                if $input_name.len() != len {
                    return Err($crate::KandError::LengthMismatch);
                }
                if $input_name.null_count() > 0 {
                    return Err($crate::KandError::InvalidData);
                }
            )+

            let lookback = lookback($($lb_param),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            $(
                let len_bytes = len * size_of::<$output_type>();
                let (ptr, $output_name) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
                let $output_name = ($output_name, unsafe {
                    std::slice::from_raw_parts_mut(ptr as *mut $output_type, len)
                });

                if std::any::TypeId::of::<$output_type>() == std::any::TypeId::of::<$crate::TAFloat>() {
                    let slice = unsafe {
                        std::slice::from_raw_parts_mut(ptr as *mut $crate::TAFloat, len)
                    };
                    slice.fill($crate::TAFloat::NAN);
                }
            )+

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

            let first_input = [ $( $input_name ),+ ][0];
            let len = first_input.len();

            $(
                if $input_name.len() != len {
                    return Err($crate::KandError::LengthMismatch);
                }
                if $input_name.null_count() > 0 {
                    return Err($crate::KandError::InvalidData);
                }
            )+

            let lookback = lookback($($lb_param),*)?;
            if len <= lookback {
                return Err($crate::KandError::InsufficientData);
            }

            $(
                let $input_name = &$input_name.values()[$input_name.offset()..];
            )+

            let len_bytes = len * size_of::<$crate::TAInt>();
            let (ptr, buffer) = $crate::helper::buffer_pool::create_pooled_buffer(len_bytes);
            let output_slice = unsafe {
                std::slice::from_raw_parts_mut(ptr as *mut $crate::TAInt, len)
            };

            $raw_fn($($input_name,)+ $($param_name,)* output_slice);

            Ok($crate::ta::types::TAArrowArray::new(buffer.into(), None))
        }
    };
}

#[macro_export]
macro_rules! kand_indicator {
    (
        $name:ident,
        type: recursive,
        inputs: { $($input_name:ident : $input_type:ty),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        state: { $($state_field:ident : $state_type:ty),* },
        init: |$($init_param:ident),*| $init_block:block,
        next: |$next_state:ident, ($($next_input_arg:ident),*)| $next_block:block
    ) => {
        $crate::paste::paste! {
            #[derive(Clone)]
            pub struct [<Stateful $name:upper>] {
                $(pub $param_name : $param_type,)*
                $(pub $state_field : $state_type,)*
                pub __kand_count: usize,
            }

            impl [<Stateful $name:upper>] {
                pub fn new($($param_name : $param_type),*) -> Result<Self, $crate::KandError> {
                    $( let $init_param = $param_name; )*
                    let ($($state_field),*) = $init_block;
                    Ok(Self {
                        $($param_name,)*
                        $($state_field,)*
                        __kand_count: 0,
                    })
                }
            }

            impl $crate::ta::traits::Indicator for [<Stateful $name:upper>] {
                type Input = ($($input_type,)+);
                type Output = $crate::TAFloat;

                fn next(&mut self, input: Self::Input) -> Result<Self::Output, $crate::KandError> {
                    self.__kand_count += 1;
                    let ($($next_input_arg,)+) = input;
                    let $next_state = self;
                    $next_block
                }

                #[cfg(feature = "arrow")]
                fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    use arrow::datatypes::{DataType, Field, Schema};
                    use std::sync::Arc;
                    let schema = Arc::new(Schema::new(vec![
                        Field::new("__kand_count", DataType::UInt64, false),
                        $(Field::new(concat!("__kand_", stringify!($param_name)), DataType::Float64, false),)*
                        $(Field::new(concat!("__kand_", stringify!($state_field)), DataType::Float64, false),)*
                    ]));
                    let mut columns: Vec<Arc<dyn arrow::array::Array>> = vec![ Arc::new(UInt64Array::from(vec![self.__kand_count as u64])) ];
                    $( columns.push(Arc::new(Float64Array::from(vec![self.$param_name as f64]))); )*
                    $( columns.push(Arc::new(Float64Array::from(vec![self.$state_field as f64]))); )*
                    arrow::record_batch::RecordBatch::try_new(schema, columns).map_err(|_| $crate::KandError::InvalidData)
                }

                #[cfg(feature = "arrow")]
                fn restore_from_record_batch(&mut self, batch: &arrow::record_batch::RecordBatch) -> Result<(), $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    let mut col = 0;
                    self.__kand_count = batch.column(col).as_any().downcast_ref::<UInt64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as usize; col += 1;
                    $( self.$param_name = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as $param_type; col += 1; )*
                    $( self.$state_field = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as $state_type; col += 1; )*
                    let _ = col;
                    Ok(())
                }
            }

            #[cfg(feature = "arrow")]
            pub struct [<Batch $name:upper>] {
                $(pub $param_name : $param_type,)*
                pub num_streams: usize,
                pub __kand_counts: Vec<usize>,
                $( pub [<__kand_ $state_field s>]: arrow_buffer::MutableBuffer, )*
            }

            #[cfg(feature = "arrow")]
            impl [<Batch $name:upper>] {
                pub fn new($($param_name : $param_type,)* num_streams: usize) -> Result<Self, $crate::KandError> {
                    use std::mem::size_of;
                    $(
                        let mut [<__kand_ $state_field s>] = arrow_buffer::MutableBuffer::new(num_streams * size_of::<$state_type>());
                        [<__kand_ $state_field s>].resize(num_streams * size_of::<$state_type>(), 0);
                    )*
                    Ok(Self {
                        $($param_name,)*
                        num_streams,
                        __kand_counts: vec![0; num_streams],
                        $( [<__kand_ $state_field s>], )*
                    })
                }
            }

            #[cfg(feature = "arrow")]
            impl Clone for [<Batch $name:upper>] {
                fn clone(&self) -> Self {
                    $(
                        let mut [<new_ $state_field s>] = arrow_buffer::MutableBuffer::new(self.[<__kand_ $state_field s>].len());
                        [<new_ $state_field s>].extend_from_slice(self.[<__kand_ $state_field s>].as_slice());
                    )*
                    Self {
                        $($param_name: self.$param_name,)*
                        num_streams: self.num_streams,
                        __kand_counts: self.__kand_counts.clone(),
                        $( [<__kand_ $state_field s>]: [<new_ $state_field s>], )*
                    }
                }
            }

            #[cfg(feature = "arrow")]
            impl $crate::ta::traits::BatchIndicator for [<Batch $name:upper>] {
                type Input = ($( $crate::id_type!($input_name, $crate::ta::types::TAArrowArray) ,)+);
                type Output = $crate::ta::types::TAArrowArray;

                fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, $crate::KandError> {
                    use std::mem::size_of;
                    use arrow::array::Array;
                    use $crate::ta::traits::Indicator;
                    let ($($input_name,)+) = input;
                    if $( $input_name.len() != self.num_streams )||* { return Err($crate::KandError::LengthMismatch); }
                    $( let [< $input_name _values>] = $input_name.values(); )*
                    $( let [< $state_field _slice>] = self.[<__kand_ $state_field s>].typed_data_mut::<$state_type>(); )*
                    let (ptr, out_buffer) = $crate::helper::buffer_pool::create_pooled_buffer(self.num_streams * size_of::<$crate::TAFloat>());
                    let output_slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut $crate::TAFloat, self.num_streams) };

                    for s in 0..self.num_streams {
                        let mut temp_state = [<Stateful $name:upper>] {
                            $($param_name: self.$param_name,)*
                            $($state_field: [< $state_field _slice>][s],)*
                            __kand_count: self.__kand_counts[s],
                        };
                        let result = temp_state.next(($( [< $input_name _values>][s], )+));
                        output_slice[s] = result?;
                        self.__kand_counts[s] = temp_state.__kand_count;
                        $( [< $state_field _slice>][s] = temp_state.$state_field; )*
                    }
                    Ok($crate::ta::types::TAArrowArray::new(out_buffer.into(), None))
                }

                fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    use arrow::datatypes::{DataType, Field, Schema};
                    use std::sync::Arc;
                    let mut fields = vec![ Field::new("__kand_count", DataType::UInt64, false) ];
                    $( fields.push(Field::new(concat!("__kand_", stringify!($param_name)), DataType::Float64, false)); )*
                    $( fields.push(Field::new(concat!("__kand_", stringify!($state_field)), DataType::Float64, false)); )*
                    let schema = Arc::new(Schema::new(fields));
                    let mut columns: Vec<Arc<dyn arrow::array::Array>> = vec![ Arc::new(UInt64Array::from(self.__kand_counts.iter().map(|&c| c as u64).collect::<Vec<_>>())) ];
                    $( columns.push(Arc::new(Float64Array::from(vec![self.$param_name as f64; self.num_streams]))); )*
                    $( columns.push(Arc::new(Float64Array::new(arrow_buffer::ScalarBuffer::new(self.[<__kand_ $state_field s>].as_slice().into(), 0, self.num_streams), None))); )*
                    arrow::record_batch::RecordBatch::try_new(schema, columns).map_err(|_| $crate::KandError::InvalidData)
                }

                fn restore_from_record_batch(&mut self, batch: &arrow::record_batch::RecordBatch) -> Result<(), $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    self.num_streams = batch.num_rows();
                    let mut col = 0;
                    let counts = batch.column(col).as_any().downcast_ref::<UInt64Array>().ok_or($crate::KandError::InvalidData)?;
                    self.__kand_counts = counts.values().iter().map(|&c| c as usize).collect();
                    col += 1;
                    $( self.$param_name = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as $param_type; col += 1; )*
                    $(
                        let state_data = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?;
                        self.[<__kand_ $state_field s>] = arrow_buffer::MutableBuffer::from_len_zeroed(state_data.len() * std::mem::size_of::<$state_type>());
                        self.[<__kand_ $state_field s>].typed_data_mut::<$state_type>().copy_from_slice(state_data.values());
                        col += 1;
                    )*
                    let _ = col;
                    Ok(())
                }
            }
        }
    };

    (
        $name:ident,
        type: sliding_window,
        inputs: { $($input_name:ident : $input_type:ty),+ },
        params: { $($param_name:ident : $param_type:ty),* },
        state: { $($state_field:ident : $state_type:ty),* },
        init: |$($init_param:ident),*| $init_block:block,
        next: |$next_state:ident, ($($next_input_arg:ident),*)| $next_block:block
    ) => {
        $crate::paste::paste! {
            #[derive(Clone)]
            pub struct [<Stateful $name:upper>] {
                $(pub $param_name : $param_type,)*
                $(pub $state_field : $state_type,)*
                pub __kand_window: Vec<($($input_type,)+)>,
                pub __kand_cursor: usize,
                pub __kand_count: usize,
            }

            impl [<Stateful $name:upper>] {
                pub fn new($($param_name : $param_type),*) -> Result<Self, $crate::KandError> {
                    $( let $init_param = $param_name; )*
                    let ($($state_field),*) = $init_block;
                    let period = [ $($param_name as usize),* ][0]; 
                    Ok(Self {
                        $($param_name,)*
                        $($state_field,)*
                        __kand_window: vec![($(0.0 as $input_type,)+); period],
                        __kand_cursor: 0,
                        __kand_count: 0,
                    })
                }
            }

            impl $crate::ta::traits::Indicator for [<Stateful $name:upper>] {
                type Input = ($($input_type,)+);
                type Output = $crate::TAFloat;

                fn next(&mut self, input: Self::Input) -> Result<Self::Output, $crate::KandError> {
                    self.__kand_count += 1;
                    let ($($next_input_arg,)+) = input;
                    let $next_state = self;
                    $next_block
                }

                #[cfg(feature = "arrow")]
                fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    use arrow::datatypes::{DataType, Field, Schema};
                    use std::sync::Arc;
                    let schema = Arc::new(Schema::new(vec![
                        Field::new("__kand_count", DataType::UInt64, false),
                        Field::new("__kand_cursor", DataType::UInt64, false),
                        $(Field::new(concat!("__kand_", stringify!($param_name)), DataType::Float64, false),)*
                        $(Field::new(concat!("__kand_", stringify!($state_field)), DataType::Float64, false),)*
                        Field::new("__kand_window", DataType::Float64, false),
                    ]));
                    let mut columns: Vec<Arc<dyn arrow::array::Array>> = vec![
                        Arc::new(UInt64Array::from(vec![self.__kand_count as u64])),
                        Arc::new(UInt64Array::from(vec![self.__kand_cursor as u64])),
                    ];
                    $( columns.push(Arc::new(Float64Array::from(vec![self.$param_name as f64]))); )*
                    $( columns.push(Arc::new(Float64Array::from(vec![self.$state_field as f64]))); )*
                    
                    let window_data: Vec<f64> = self.__kand_window.iter().map(|val| {
                        let _dummy = ($(stringify!($input_name)),*);
                        val.0 as f64
                    }).collect();
                    columns.push(Arc::new(Float64Array::from(window_data)));
                    arrow::record_batch::RecordBatch::try_new(schema, columns).map_err(|_| $crate::KandError::InvalidData)
                }

                #[cfg(feature = "arrow")]
                fn restore_from_record_batch(&mut self, batch: &arrow::record_batch::RecordBatch) -> Result<(), $crate::KandError> {
                    use arrow::array::{Float64Array, UInt64Array};
                    let mut col = 0;
                    self.__kand_count = batch.column(col).as_any().downcast_ref::<UInt64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as usize; col += 1;
                    self.__kand_cursor = batch.column(col).as_any().downcast_ref::<UInt64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as usize; col += 1;
                    $( self.$param_name = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as $param_type; col += 1; )*
                    $( self.$state_field = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as $state_type; col += 1; )*
                    let window_data = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?;
                    self.__kand_window = window_data.values().iter().map(|&x| ( $( x as $crate::id_type!($input_name, $input_type) ,)+ )).collect();
                    let _ = col;
                    Ok(())
                }
            }

            #[cfg(feature = "arrow")]
            pub struct [<Batch $name:upper>] {
                $(pub $param_name : $param_type,)*
                pub num_streams: usize,
                pub __kand_counts: Vec<usize>,
                pub __kand_cursors: Vec<usize>,
                pub __kand_windows: arrow_buffer::MutableBuffer,
                $( pub [<__kand_ $state_field s>]: arrow_buffer::MutableBuffer, )*
            }

            #[cfg(feature = "arrow")]
            impl [<Batch $name:upper>] {
                pub fn new($($param_name : $param_type,)* num_streams: usize) -> Result<Self, $crate::KandError> {
                    use std::mem::size_of;
                    let period = [ $($param_name as usize),* ][0]; 
                    $(
                        let mut [<__kand_ $state_field s>] = arrow_buffer::MutableBuffer::new(num_streams * size_of::<$state_type>());
                        [<__kand_ $state_field s>].resize(num_streams * size_of::<$state_type>(), 0);
                    )*
                    let mut __kand_windows = arrow_buffer::MutableBuffer::new(num_streams * period * size_of::<$crate::TAFloat>());
                    __kand_windows.resize(num_streams * period * size_of::<$crate::TAFloat>(), 0);
                    Ok(Self {
                        $($param_name,)*
                        num_streams,
                        __kand_counts: vec![0; num_streams],
                        __kand_cursors: vec![0; num_streams],
                        __kand_windows,
                        $( [<__kand_ $state_field s>], )*
                    })
                }
            }

            #[cfg(feature = "arrow")]
            impl Clone for [<Batch $name:upper>] {
                fn clone(&self) -> Self {
                    let mut new_windows = arrow_buffer::MutableBuffer::new(self.__kand_windows.len());
                    new_windows.extend_from_slice(self.__kand_windows.as_slice());
                    $(
                        let mut [<new_ $state_field s>] = arrow_buffer::MutableBuffer::new(self.[<__kand_ $state_field s>].len());
                        [<new_ $state_field s>].extend_from_slice(self.[<__kand_ $state_field s>].as_slice());
                    )*
                    Self {
                        $($param_name: self.$param_name,)*
                        num_streams: self.num_streams,
                        __kand_counts: self.__kand_counts.clone(),
                        __kand_cursors: self.__kand_cursors.clone(),
                        __kand_windows: new_windows,
                        $( [<__kand_ $state_field s>]: [<new_ $state_field s>], )*
                    }
                }
            }

            #[cfg(feature = "arrow")]
            impl $crate::ta::traits::BatchIndicator for [<Batch $name:upper>] {
                type Input = ($( $crate::id_type!($input_name, $crate::ta::types::TAArrowArray) ,)+);
                type Output = $crate::ta::types::TAArrowArray;

                fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, $crate::KandError> {
                    use std::mem::size_of;
                    use arrow::array::Array;
                    use $crate::ta::traits::Indicator;
                    let ($($input_name,)+) = input;
                    if $( $input_name.len() != self.num_streams )||* { return Err($crate::KandError::LengthMismatch); }
                    let period = [ $(self.$param_name as usize),* ][0]; 
                    $( let [< $input_name _values>] = $input_name.values(); )*
                    let windows_slice = self.__kand_windows.typed_data_mut::<$crate::TAFloat>();
                    $( let [< $state_field _slice>] = self.[<__kand_ $state_field s>].typed_data_mut::<$state_type>(); )*
                    let (ptr, out_buffer) = $crate::helper::buffer_pool::create_pooled_buffer(self.num_streams * size_of::<$crate::TAFloat>());
                    let output_slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut $crate::TAFloat, self.num_streams) };

                    for s in 0..self.num_streams {
                        let mut temp_state = [<Stateful $name:upper>] {
                            $($param_name: self.$param_name,)*
                            $($state_field: [< $state_field _slice>][s],)*
                            __kand_window: windows_slice[s * period..(s+1) * period].iter().map(|&x| ( $( x as $crate::id_type!($input_name, $input_type) ,)+ )).collect(),
                            __kand_cursor: self.__kand_cursors[s],
                            __kand_count: self.__kand_counts[s],
                        };
                        let result = temp_state.next(($( [< $input_name _values>][s], )+));
                        output_slice[s] = result?;
                        self.__kand_counts[s] = temp_state.__kand_count;
                        self.__kand_cursors[s] = temp_state.__kand_cursor;
                        for (i, val) in temp_state.__kand_window.into_iter().enumerate() { 
                            let _dummy = ($(stringify!($input_name)),*);
                            windows_slice[s * period + i] = val.0 as f64; 
                        }
                        $( [< $state_field _slice>][s] = temp_state.$state_field; )*
                    }
                    Ok($crate::ta::types::TAArrowArray::new(out_buffer.into(), None))
                }

                fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, $crate::KandError> {
                    use arrow::array::{FixedSizeListArray, Float64Array, UInt64Array};
                    use arrow::datatypes::{DataType, Field, Schema};
                    use std::sync::Arc;
                    let period = [ $(self.$param_name as usize),* ][0]; 
                    let mut fields = vec![ Field::new("__kand_count", DataType::UInt64, false), Field::new("__kand_cursor", DataType::UInt64, false) ];
                    $( fields.push(Field::new(concat!("__kand_", stringify!($param_name)), DataType::Float64, false)); )*
                    $( fields.push(Field::new(concat!("__kand_", stringify!($state_field)), DataType::Float64, false)); )*
                    fields.push(Field::new("__kand_window", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, true)), period as i32), false));
                    let schema = Arc::new(Schema::new(fields));
                    let mut columns: Vec<Arc<dyn arrow::array::Array>> = vec![
                        Arc::new(UInt64Array::from(self.__kand_counts.iter().map(|&c| c as u64).collect::<Vec<_>>())),
                        Arc::new(UInt64Array::from(self.__kand_cursors.iter().map(|&c| c as u64).collect::<Vec<_>>())),
                    ];
                    $( columns.push(Arc::new(Float64Array::from(vec![self.$param_name as f64; self.num_streams]))); )*
                    $( columns.push(Arc::new(Float64Array::new(arrow_buffer::ScalarBuffer::new(self.[<__kand_ $state_field s>].as_slice().into(), 0, self.num_streams), None))); )*
                    let windows_data = Float64Array::new(arrow_buffer::ScalarBuffer::new(self.__kand_windows.as_slice().into(), 0, self.num_streams * period), None);
                    columns.push(Arc::new(FixedSizeListArray::new(Arc::new(Field::new("item", DataType::Float64, true)), period as i32, Arc::new(windows_data), None)));
                    arrow::record_batch::RecordBatch::try_new(schema, columns).map_err(|_| $crate::KandError::InvalidData)
                }

                fn restore_from_record_batch(&mut self, batch: &arrow::record_batch::RecordBatch) -> Result<(), $crate::KandError> {
                    use arrow::array::{FixedSizeListArray, Float64Array, UInt64Array};
                    self.num_streams = batch.num_rows();
                    let mut col = 0;
                    self.__kand_counts = batch.column(col).as_any().downcast_ref::<UInt64Array>().ok_or($crate::KandError::InvalidData)?.values().iter().map(|&c| c as usize).collect(); col += 1;
                    self.__kand_cursors = batch.column(col).as_any().downcast_ref::<UInt64Array>().ok_or($crate::KandError::InvalidData)?.values().iter().map(|&c| c as usize).collect(); col += 1;
                    $( self.$param_name = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?.value(0) as $param_type; col += 1; )*
                    $(
                        let state_data = batch.column(col).as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?;
                        self.[<__kand_ $state_field s>] = arrow_buffer::MutableBuffer::from_len_zeroed(state_data.len() * std::mem::size_of::<$state_type>());
                        self.[<__kand_ $state_field s>].typed_data_mut::<$state_type>().copy_from_slice(state_data.values());
                        col += 1;
                    )*
                    let windows_list = batch.column(col).as_any().downcast_ref::<FixedSizeListArray>().ok_or($crate::KandError::InvalidData)?;
                    let windows_data = windows_list.values().as_any().downcast_ref::<Float64Array>().ok_or($crate::KandError::InvalidData)?;
                    self.__kand_windows = arrow_buffer::MutableBuffer::from_len_zeroed(windows_data.len() * std::mem::size_of::<$crate::TAFloat>());
                    self.__kand_windows.typed_data_mut::<$crate::TAFloat>().copy_from_slice(windows_data.values());
                    let _ = col;
                    Ok(())
                }
            }
        }
    };
}
