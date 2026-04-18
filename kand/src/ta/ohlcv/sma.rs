use crate::{KandError, TAFloat, TAPeriod};

/// Returns the lookback period required for Simple Moving Average (SMA).
///
/// # Arguments
/// * `opt_period` - The number of periods to average.
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period (period - 1).
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2.
pub const fn lookback(opt_period: TAPeriod) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period - 1)
}

/// Stateful implementation of Simple Moving Average (SMA).
pub struct StatefulSMA {
    period: TAPeriod,
    window: Vec<TAFloat>,
    sum: TAFloat,
}

impl StatefulSMA {
    /// Creates a new StatefulSMA instance.
    pub fn new(period: TAPeriod) -> Result<Self, KandError> {
        #[cfg(feature = "check")]
        {
            if period < 2 {
                return Err(KandError::InvalidParameter);
            }
        }
        Ok(Self {
            period,
            window: Vec::with_capacity(period),
            sum: 0.0,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulSMA {
    type Input = TAFloat;
    type Output = TAFloat;

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        if self.window.len() < self.period {
            self.window.push(input);
            self.sum += input;
            if self.window.len() == self.period {
                Ok(self.sum / self.period as TAFloat)
            } else {
                Ok(TAFloat::NAN)
            }
        } else {
            let old_val = self.window[0];
            self.window.rotate_left(1);
            self.window[self.period - 1] = input;
            self.sum += input - old_val;
            Ok(self.sum / self.period as TAFloat)
        }
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{Float64Array, UInt64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("period", DataType::UInt64, false),
            Field::new("sum", DataType::Float64, false),
            Field::new("window", DataType::Float64, false),
        ]));

        let period_arr = UInt64Array::from(vec![self.period as u64]);
        let sum_arr = Float64Array::from(vec![self.sum]);
        let window_arr = Float64Array::from(self.window.clone());

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                Arc::new(period_arr),
                Arc::new(sum_arr),
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
        let sum = batch
            .column(1)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);
        let window = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;

        self.period = period;
        self.sum = sum;
        self.window = window.values().to_vec();

        Ok(())
    }
}

/// Vectorized implementation of Simple Moving Average (SMA) for multiple independent streams.
#[cfg(feature = "arrow")]
pub struct BatchSMA {
    period: TAPeriod,
    num_streams: usize,
    // Buffer storing the last 'period' values for each stream.
    // Layout: [s0_t0, s0_t1, ..., s0_tP-1, s1_t0, ...]
    windows: arrow_buffer::MutableBuffer,
    // Buffer storing the current sum for each stream.
    sums: arrow_buffer::MutableBuffer,
    // Current index in the circular windows buffer.
    cursor: usize,
    // Number of values received so far (to handle initial NaN period).
    count: usize,
}

#[cfg(feature = "arrow")]
impl BatchSMA {
    /// Creates a new BatchSMA instance for a fixed number of streams.
    pub fn new(period: TAPeriod, num_streams: usize) -> Result<Self, KandError> {
        use std::mem::size_of;
        #[cfg(feature = "check")]
        {
            if period < 2 || num_streams == 0 {
                return Err(KandError::InvalidParameter);
            }
        }

        let mut windows =
            arrow_buffer::MutableBuffer::new(num_streams * period * size_of::<TAFloat>());
        windows.resize(num_streams * period * size_of::<TAFloat>(), 0);

        let mut sums = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        sums.resize(num_streams * size_of::<TAFloat>(), 0);

        Ok(Self {
            period,
            num_streams,
            windows,
            sums,
            cursor: 0,
            count: 0,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchSMA {
    type Input = crate::ta::types::TAArrowArray;
    type Output = crate::ta::types::TAArrowArray;

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        use arrow::array::Array;
        use std::mem::size_of;

        if input.len() != self.num_streams {
            return Err(KandError::LengthMismatch);
        }

        if input.null_count() > 0 {
            return Err(KandError::InvalidData);
        }

        let input_values = input.values();
        let sums_slice = self.sums.typed_data_mut::<TAFloat>();
        let windows_slice = self.windows.typed_data_mut::<TAFloat>();

        let (ptr, out_buffer) = crate::helper::buffer_pool::create_pooled_buffer(
            self.num_streams * size_of::<TAFloat>(),
        );
        let output_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut TAFloat, self.num_streams) };

        self.count += 1;
        let is_valid = self.count >= self.period;

        for s in 0..self.num_streams {
            let val = input_values[s];
            let window_offset = s * self.period + self.cursor;
            let old_val = windows_slice[window_offset];

            windows_slice[window_offset] = val;

            if self.count <= self.period {
                sums_slice[s] += val;
            } else {
                sums_slice[s] += val - old_val;
            }

            if is_valid {
                output_slice[s] = sums_slice[s] / self.period as TAFloat;
            } else {
                output_slice[s] = TAFloat::NAN;
            }
        }

        self.cursor = (self.cursor + 1) % self.period;

        Ok(crate::ta::types::TAArrowArray::new(out_buffer.into(), None))
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{FixedSizeListArray, Float64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new_with_metadata(
            vec![
                Field::new("sum", DataType::Float64, false),
                Field::new(
                    "window",
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float64, true)),
                        self.period as i32,
                    ),
                    false,
                ),
            ],
            std::collections::HashMap::from([
                ("period".to_string(), self.period.to_string()),
                ("cursor".to_string(), self.cursor.to_string()),
                ("count".to_string(), self.count.to_string()),
            ]),
        ));

        let sums_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(self.sums.as_slice().into(), 0, self.num_streams),
            None,
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

        arrow::record_batch::RecordBatch::try_new(schema, vec![sums_arr, windows_arr])
            .map_err(|_| KandError::InvalidData)
    }

    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(
        &mut self,
        batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), KandError> {
        use arrow::array::{FixedSizeListArray, Float64Array};

        let period = batch
            .schema()
            .metadata()
            .get("period")
            .ok_or(KandError::InvalidData)?
            .parse()
            .map_err(|_| KandError::InvalidData)?;
        let cursor = batch
            .schema()
            .metadata()
            .get("cursor")
            .ok_or(KandError::InvalidData)?
            .parse()
            .map_err(|_| KandError::InvalidData)?;
        let count = batch
            .schema()
            .metadata()
            .get("count")
            .ok_or(KandError::InvalidData)?
            .parse()
            .map_err(|_| KandError::InvalidData)?;

        let num_streams = batch.num_rows();

        let sums = batch
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;
        let windows_list = batch
            .column(1)
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
        self.cursor = cursor;
        self.count = count;

        self.sums = arrow_buffer::MutableBuffer::from_len_zeroed(
            sums.len() * std::mem::size_of::<TAFloat>(),
        );
        self.sums
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(sums.values());

        self.windows = arrow_buffer::MutableBuffer::from_len_zeroed(
            windows.len() * std::mem::size_of::<TAFloat>(),
        );
        self.windows
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(windows.values());

        Ok(())
    }
}

/// Calculates SMA without input validation.
pub fn sma_raw(input: &[TAFloat], opt_period: TAPeriod, output: &mut [TAFloat]) {
    let mut sum = 0.0;
    for val in input.iter().take(opt_period) {
        sum += val;
    }

    output[opt_period - 1] = sum / opt_period as TAFloat;

    for i in opt_period..input.len() {
        sum += input[i] - input[i - opt_period];
        output[i] = sum / opt_period as TAFloat;
    }
}

/// Calculates the Simple Moving Average (SMA) for a given data slice.
///
/// # Description
/// SMA is the unweighted mean of the previous `n` data points.
///
/// # Arguments
/// * `input` - The data slice to calculate SMA for.
/// * `opt_period` - The number of periods to average.
/// * `output` - The output slice for SMA values.
///
/// # Returns
/// * `Result<(), KandError>` - `Ok(())` on success.
///
/// # Errors
/// * `KandError::InvalidData` - If input is empty.
/// * `KandError::LengthMismatch` - If input and output lengths differ.
/// * `KandError::InsufficientData` - If input length is less than `opt_period`.
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2.
pub fn sma(
    input: &[TAFloat],
    opt_period: TAPeriod,
    output: &mut [TAFloat],
) -> Result<(), KandError> {
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
        for i in 0..len {
            if input[i].is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    sma_raw(input, opt_period, output);

    // Initial values
    for val in output.iter_mut().take(lookback) {
        *val = TAFloat::NAN;
    }

    Ok(())
}

/// Calculates SMA incrementally for a single value without validation.
#[inline]
pub fn sma_inc_raw(
    input: TAFloat,
    prev_input: TAFloat,
    prev_sma: TAFloat,
    opt_period: TAPeriod,
) -> TAFloat {
    prev_sma + (input - prev_input) / opt_period as TAFloat
}

/// Calculates SMA incrementally for a single value.
///
/// # Arguments
/// * `input` - Current value.
/// * `prev_input` - Value from `opt_period` ago.
/// * `prev_sma` - Previous SMA value.
/// * `opt_period` - The number of periods to average.
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The new SMA value.
pub fn sma_inc(
    input: TAFloat,
    prev_input: TAFloat,
    prev_sma: TAFloat,
    opt_period: TAPeriod,
) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        if input.is_nan() || prev_input.is_nan() || prev_sma.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(prev_sma + (input - prev_input) / opt_period as TAFloat)
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    sma_arrow,
    crate::ta::ohlcv::sma::sma_raw,
    inputs: { input },
    params: { opt_period: TAPeriod },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    use super::*;

    #[test]
    fn test_stateful_sma() {
        let mut sma = StatefulSMA::new(3).unwrap();
        assert!(sma.next(10.0).unwrap().is_nan());
        assert!(sma.next(11.0).unwrap().is_nan());
        assert_relative_eq!(sma.next(12.0).unwrap(), 11.0);
        assert_relative_eq!(sma.next(13.0).unwrap(), 12.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_sma() {
        use crate::ta::traits::BatchIndicator;
        let mut batch_sma = BatchSMA::new(3, 2).unwrap();

        // t0
        let input = TAArrowArray::from(vec![10.0, 20.0]);
        let out = batch_sma.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t1
        let input = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_sma.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t2 - first valid
        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_sma.next_batch(input).unwrap();
        assert_relative_eq!(out.value(0), 11.0);
        assert_relative_eq!(out.value(1), 21.0);

        // t3
        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let out = batch_sma.next_batch(input).unwrap();
        assert_relative_eq!(out.value(0), 12.0);
        assert_relative_eq!(out.value(1), 22.0);

        // Test persistence
        let batch = batch_sma.to_record_batch().unwrap();
        let mut new_batch_sma = BatchSMA::new(3, 2).unwrap();
        new_batch_sma.from_record_batch(&batch).unwrap();

        // t4
        let input = TAArrowArray::from(vec![14.0, 24.0]);
        let out = new_batch_sma.next_batch(input).unwrap();
        // sum = 12 + 13 + 14 = 39 / 3 = 13.0
        assert_relative_eq!(out.value(0), 13.0);
        assert_relative_eq!(out.value(1), 23.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_sma() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let input_arrow = TAArrowArray::from(input);
        let period = 3;

        let result = sma_arrow(&input_arrow, period).unwrap();

        assert_eq!(result.len(), 5);
        for i in 0..period - 1 {
            assert!(result.value(i).is_nan());
        }
        assert_relative_eq!(result.value(2), 2.0);
        assert_relative_eq!(result.value(3), 3.0);
        assert_relative_eq!(result.value(4), 4.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_sma_arrow() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let input_arrow = TAArrowArray::from(input);
        const PERIOD: usize = 3;

        let result = sma_arrow(&input_arrow, PERIOD).unwrap();

        assert_eq!(result.len(), 5);
        for i in 0..PERIOD - 1 {
            assert!(result.value(i).is_nan());
        }
        assert_relative_eq!(result.value(2), 2.0);
        assert_relative_eq!(result.value(3), 3.0);
        assert_relative_eq!(result.value(4), 4.0);
    }
}
