use crate::{KandError, TAFloat, TAPeriod};

/// Returns the lookback period for Rate of Change (ROC) calculation without input validation.
#[inline]
pub const fn lookback_raw(opt_period: usize) -> TAPeriod {
    opt_period
}

/// Returns the lookback period required for ROC calculation.
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 1 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(lookback_raw(opt_period))
}

/// Stateful implementation of Rate of Change (ROC).
#[derive(Clone)]
pub struct StatefulROC {
    period: usize,
    count: usize,
    window: Vec<TAFloat>,
    cursor: usize,
}

impl StatefulROC {
    /// Creates a new StatefulROC instance.
    pub fn new(period: usize) -> Result<Self, KandError> {
        #[cfg(feature = "check")]
        {
            if period < 1 {
                return Err(KandError::InvalidParameter);
            }
        }
        Ok(Self {
            period,
            count: 0,
            window: vec![0.0; period],
            cursor: 0,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulROC {
    type Input = TAFloat;
    type Output = TAFloat;

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        self.count += 1;
        let old_val = self.window[self.cursor];
        self.window[self.cursor] = input;
        self.cursor = (self.cursor + 1) % self.period;

        if self.count <= self.period {
            Ok(TAFloat::NAN)
        } else if old_val == 0.0 {
            Ok(TAFloat::NAN) // Division by zero
        } else {
            Ok(((input - old_val) / old_val) * 100.0)
        }
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{Float64Array, UInt64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("__kand_period", DataType::UInt64, false),
            Field::new("__kand_count", DataType::UInt64, false),
            Field::new("__kand_cursor", DataType::UInt64, false),
            Field::new("__kand_window", DataType::Float64, false),
        ]));

        let period_arr = UInt64Array::from(vec![self.period as u64]);
        let count_arr = UInt64Array::from(vec![self.count as u64]);
        let cursor_arr = UInt64Array::from(vec![self.cursor as u64]);
        let window_arr = Float64Array::from(self.window.clone());

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                Arc::new(period_arr),
                Arc::new(count_arr),
                Arc::new(cursor_arr),
                Arc::new(window_arr),
            ],
        )
        .map_err(|_| KandError::InvalidData)
    }

    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(
        &mut self,
        batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), KandError> {
        use arrow::array::{Float64Array, UInt64Array};

        let period = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0) as usize;
        let count = batch
            .column(1)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0) as usize;
        let cursor = batch
            .column(2)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0) as usize;
        let window = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;

        self.period = period;
        self.count = count;
        self.cursor = cursor;
        self.window = window.values().to_vec();

        Ok(())
    }
}

/// Vectorized implementation of Rate of Change (ROC) for multiple independent streams.
#[cfg(feature = "arrow")]
#[derive(Clone)]
pub struct BatchROC {
    period: usize,
    num_streams: usize,
    counts: Vec<usize>,
    cursors: Vec<usize>,
    windows: arrow_buffer::MutableBuffer,
}

#[cfg(feature = "arrow")]
impl BatchROC {
    /// Creates a new BatchROC instance.
    pub fn new(period: usize, num_streams: usize) -> Result<Self, KandError> {
        use std::mem::size_of;
        #[cfg(feature = "check")]
        {
            if period < 1 || num_streams == 0 {
                return Err(KandError::InvalidParameter);
            }
        }

        let mut windows = arrow_buffer::MutableBuffer::new(num_streams * period * size_of::<TAFloat>());
        windows.resize(num_streams * period * size_of::<TAFloat>(), 0);

        Ok(Self {
            period,
            num_streams,
            counts: vec![0; num_streams],
            cursors: vec![0; num_streams],
            windows,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchROC {
    type Input = crate::ta::types::TAArrowArray;
    type Output = crate::ta::types::TAArrowArray;

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        use std::mem::size_of;

        if input.len() != self.num_streams {
            return Err(KandError::LengthMismatch);
        }

        let input_values = input.values();
        let windows_slice = self.windows.typed_data_mut::<TAFloat>();

        let (ptr, out_buffer) = crate::helper::buffer_pool::create_pooled_buffer(
            self.num_streams * size_of::<TAFloat>(),
        );
        let output_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut TAFloat, self.num_streams) };

        for s in 0..self.num_streams {
            let val = input_values[s];
            self.counts[s] += 1;

            let window_offset = s * self.period + self.cursors[s];
            let old_val = windows_slice[window_offset];
            windows_slice[window_offset] = val;
            self.cursors[s] = (self.cursors[s] + 1) % self.period;

            if self.counts[s] <= self.period || old_val == 0.0 {
                output_slice[s] = TAFloat::NAN;
            } else {
                output_slice[s] = ((val - old_val) / old_val) * 100.0;
            }
        }

        Ok(crate::ta::types::TAArrowArray::new(out_buffer.into(), None))
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{FixedSizeListArray, Float64Array, UInt64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("__kand_period", DataType::UInt64, false),
            Field::new("__kand_count", DataType::UInt64, false),
            Field::new("__kand_cursor", DataType::UInt64, false),
            Field::new(
                "__kand_window",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float64, true)),
                    self.period as i32,
                ),
                false,
            ),
        ]));

        let period_arr = Arc::new(UInt64Array::from(vec![self.period as u64; self.num_streams]))
            as Arc<dyn arrow::array::Array>;
        let counts_arr = Arc::new(UInt64Array::from(
            self.counts.iter().map(|&c| c as u64).collect::<Vec<_>>(),
        )) as Arc<dyn arrow::array::Array>;
        let cursors_arr = Arc::new(UInt64Array::from(
            self.cursors.iter().map(|&c| c as u64).collect::<Vec<_>>(),
        )) as Arc<dyn arrow::array::Array>;

        let windows_data = Float64Array::new(
            arrow_buffer::ScalarBuffer::new(
                self.windows.as_slice().into(),
                0,
                self.num_streams * self.period,
            ),
            None,
        );
        let windows_arr = Arc::new(FixedSizeListArray::new(
            Arc::new(Field::new("item", DataType::Float64, true)),
            self.period as i32,
            Arc::new(windows_data),
            None,
        )) as Arc<dyn arrow::array::Array>;

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![period_arr, counts_arr, cursors_arr, windows_arr],
        )
        .map_err(|_| KandError::InvalidData)
    }

    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(
        &mut self,
        batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), KandError> {
        use arrow::array::{FixedSizeListArray, Float64Array, UInt64Array};

        let num_streams = batch.num_rows();
        if num_streams == 0 {
            return Err(KandError::InvalidData);
        }

        let period = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0) as usize;
        let counts = batch
            .column(1)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(KandError::InvalidData)?;
        let cursors = batch
            .column(2)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(KandError::InvalidData)?;
        let windows_list = batch
            .column(3)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .ok_or(KandError::InvalidData)?;
        let windows = windows_list
            .values()
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;

        self.period = period;
        self.num_streams = num_streams;
        self.counts = counts.values().iter().map(|&c| c as usize).collect();
        self.cursors = cursors.values().iter().map(|&c| c as usize).collect();

        self.windows = arrow_buffer::MutableBuffer::from_len_zeroed(
            windows.len() * std::mem::size_of::<TAFloat>(),
        );
        self.windows
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(windows.values());

        Ok(())
    }
}

/// Calculates ROC without input validation.
pub fn roc_raw(input: &[TAFloat], opt_period: usize, output: &mut [TAFloat]) {
    for i in opt_period..input.len() {
        let old_val = input[i - opt_period];
        if old_val != 0.0 {
            output[i] = ((input[i] - old_val) / old_val) * 100.0;
        } else {
            output[i] = TAFloat::NAN;
        }
    }
}

/// Calculates the Rate of Change (ROC) for a price series.
pub fn roc(input: &[TAFloat], opt_period: usize, output: &mut [TAFloat]) -> Result<(), KandError> {
    let len = input.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        if len == 0 {
            return Err(KandError::InvalidData);
        }
        if len != output.len() {
            return Err(KandError::LengthMismatch);
        }
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input {
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    roc_raw(input, opt_period, output);

    // Initial values
    output[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Calculates ROC incrementally without validation.
#[inline]
pub fn roc_inc_raw(input_current_price: TAFloat, input_old_price: TAFloat) -> TAFloat {
    if input_old_price != 0.0 {
        ((input_current_price - input_old_price) / input_old_price) * 100.0
    } else {
        TAFloat::NAN
    }
}

/// Calculates ROC incrementally for a single value.
pub fn roc_inc(input_current_price: TAFloat, input_old_price: TAFloat) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check-nan")]
    {
        if input_current_price.is_nan() || input_old_price.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }
    Ok(roc_inc_raw(input_current_price, input_old_price))
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    roc_arrow,
    crate::ta::ohlcv::roc::roc_raw,
    inputs: { input },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    #[test]
    fn test_roc_calculation() {
        let prices = vec![1.0, 2.0, 5.0, 4.0, 8.0];
        let period = 2;
        let mut output = vec![0.0; 5];

        roc(&prices, period, &mut output).unwrap();

        assert!(output[0].is_nan());
        assert!(output[1].is_nan());
        assert_relative_eq!(output[2], 400.0, epsilon = 0.0001);
        assert_relative_eq!(output[3], 100.0, epsilon = 0.0001);
        assert_relative_eq!(output[4], 60.0, epsilon = 0.0001);
    }

    #[test]
    fn test_stateful_roc() {
        let mut roc_state = StatefulROC::new(2).unwrap();
        assert!(roc_state.next(1.0).unwrap().is_nan());
        assert!(roc_state.next(2.0).unwrap().is_nan());
        assert_relative_eq!(roc_state.next(5.0).unwrap(), 400.0);
        assert_relative_eq!(roc_state.next(4.0).unwrap(), 100.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_roc() {
        let mut batch_roc = BatchROC::new(2, 2).unwrap();

        // t0
        let input = TAArrowArray::from(vec![1.0, 10.0]);
        let out = batch_roc.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![2.0, 12.0]);
        let out = batch_roc.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![5.0, 15.0]);
        let out = batch_roc.next_batch(input).unwrap();
        assert_relative_eq!(out.value(0), 400.0);
        assert_relative_eq!(out.value(1), 50.0);
    }
}
