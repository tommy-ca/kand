use super::trange;
use crate::{KandError, TAFloat};

/// Returns the lookback period required for ATR calculation.
///
/// # Description
/// The lookback period represents the number of data points needed before the first valid output
/// can be calculated. For ATR, this equals the specified period.
///
/// # Arguments
/// * `opt_period` - The time period used for ATR calculation (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period on success
///
/// # Errors
/// * Returns `KandError::InvalidParameter` if `opt_period` is less than 2
///
/// # Example
/// ```
/// use kand::ta::ohlcv::atr;
/// let period = 14;
/// let lookback = atr::lookback(period).unwrap();
/// assert_eq!(lookback, 14); // lookback equals period
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period)
}

/// Stateful implementation of Average True Range (ATR).
#[derive(Clone)]
pub struct StatefulATR {
    period: usize,
    count: usize,
    prev_atr: TAFloat,
    prev_close: TAFloat,
    tr_sum: TAFloat,
}

impl StatefulATR {
    /// Creates a new StatefulATR instance.
    pub fn new(period: usize) -> Result<Self, KandError> {
        #[cfg(feature = "check")]
        {
            if period < 2 {
                return Err(KandError::InvalidParameter);
            }
        }
        Ok(Self {
            period,
            count: 0,
            prev_atr: 0.0,
            prev_close: 0.0,
            tr_sum: 0.0,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulATR {
    type Input = (TAFloat, TAFloat, TAFloat); // high, low, close
    type Output = TAFloat;

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        let (high, low, close) = input;
        self.count += 1;

        if self.count == 1 {
            self.prev_close = close;
            Ok(TAFloat::NAN)
        } else if self.count <= self.period {
            let tr = super::trange::trange_inc_raw(high, low, self.prev_close);
            self.tr_sum += tr;
            self.prev_close = close;
            Ok(TAFloat::NAN)
        } else if self.count == self.period + 1 {
            let tr = super::trange::trange_inc_raw(high, low, self.prev_close);
            self.tr_sum += tr;
            self.prev_atr = self.tr_sum / self.period as TAFloat;
            self.prev_close = close;
            Ok(self.prev_atr)
        } else {
            let tr = super::trange::trange_inc_raw(high, low, self.prev_close);
            self.prev_atr =
                self.prev_atr.mul_add((self.period - 1) as TAFloat, tr) / (self.period as TAFloat);
            self.prev_close = close;
            Ok(self.prev_atr)
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
            Field::new("__kand_prev_atr", DataType::Float64, false),
            Field::new("__kand_prev_close", DataType::Float64, false),
            Field::new("__kand_tr_sum", DataType::Float64, false),
        ]));

        let period_arr = UInt64Array::from(vec![self.period as u64]);
        let count_arr = UInt64Array::from(vec![self.count as u64]);
        let prev_atr_arr = Float64Array::from(vec![self.prev_atr]);
        let prev_close_arr = Float64Array::from(vec![self.prev_close]);
        let tr_sum_arr = Float64Array::from(vec![self.tr_sum]);

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                Arc::new(period_arr),
                Arc::new(count_arr),
                Arc::new(prev_atr_arr),
                Arc::new(prev_close_arr),
                Arc::new(tr_sum_arr),
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
        let prev_atr = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);
        let prev_close = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);
        let tr_sum = batch
            .column(4)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);

        self.period = period;
        self.count = count;
        self.prev_atr = prev_atr;
        self.prev_close = prev_close;
        self.tr_sum = tr_sum;

        Ok(())
    }
}

/// Vectorized implementation of Average True Range (ATR) for multiple independent streams.
#[cfg(feature = "arrow")]
pub struct BatchATR {
    period: usize,
    num_streams: usize,
    counts: Vec<usize>,
    prev_atrs: arrow_buffer::MutableBuffer,
    prev_closes: arrow_buffer::MutableBuffer,
    tr_sums: arrow_buffer::MutableBuffer,
}

#[cfg(feature = "arrow")]
impl Clone for BatchATR {
    fn clone(&self) -> Self {
        let mut new_prev_atrs = arrow_buffer::MutableBuffer::new(self.prev_atrs.len());
        new_prev_atrs.extend_from_slice(self.prev_atrs.as_slice());
        let mut new_prev_closes = arrow_buffer::MutableBuffer::new(self.prev_closes.len());
        new_prev_closes.extend_from_slice(self.prev_closes.as_slice());
        let mut new_tr_sums = arrow_buffer::MutableBuffer::new(self.tr_sums.len());
        new_tr_sums.extend_from_slice(self.tr_sums.as_slice());

        Self {
            period: self.period,
            num_streams: self.num_streams,
            counts: self.counts.clone(),
            prev_atrs: new_prev_atrs,
            prev_closes: new_prev_closes,
            tr_sums: new_tr_sums,
        }
    }
}

#[cfg(feature = "arrow")]
impl BatchATR {
    /// Creates a new BatchATR instance.
    pub fn new(period: usize, num_streams: usize) -> Result<Self, KandError> {
        use std::mem::size_of;
        #[cfg(feature = "check")]
        {
            if period < 2 || num_streams == 0 {
                return Err(KandError::InvalidParameter);
            }
        }

        let mut prev_atrs = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        prev_atrs.resize(num_streams * size_of::<TAFloat>(), 0);

        let mut prev_closes = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        prev_closes.resize(num_streams * size_of::<TAFloat>(), 0);

        let mut tr_sums = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        tr_sums.resize(num_streams * size_of::<TAFloat>(), 0);

        Ok(Self {
            period,
            num_streams,
            counts: vec![0; num_streams],
            prev_atrs,
            prev_closes,
            tr_sums,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchATR {
    type Input = (
        crate::ta::types::TAArrowArray,
        crate::ta::types::TAArrowArray,
        crate::ta::types::TAArrowArray,
    );
    type Output = crate::ta::types::TAArrowArray;

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        use std::mem::size_of;

        let (high, low, close) = input;
        if high.len() != self.num_streams
            || low.len() != self.num_streams
            || close.len() != self.num_streams
        {
            return Err(KandError::LengthMismatch);
        }

        let high_values = high.values();
        let low_values = low.values();
        let close_values = close.values();

        let prev_atrs_slice = self.prev_atrs.typed_data_mut::<TAFloat>();
        let prev_closes_slice = self.prev_closes.typed_data_mut::<TAFloat>();
        let tr_sums_slice = self.tr_sums.typed_data_mut::<TAFloat>();

        let (ptr, out_buffer) = crate::helper::buffer_pool::create_pooled_buffer(
            self.num_streams * size_of::<TAFloat>(),
        );
        let output_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut TAFloat, self.num_streams) };

        for s in 0..self.num_streams {
            let h = high_values[s];
            let l = low_values[s];
            let c = close_values[s];
            self.counts[s] += 1;

            if self.counts[s] == 1 {
                prev_closes_slice[s] = c;
                output_slice[s] = TAFloat::NAN;
            } else if self.counts[s] <= self.period {
                let tr = super::trange::trange_inc_raw(h, l, prev_closes_slice[s]);
                tr_sums_slice[s] += tr;
                prev_closes_slice[s] = c;
                output_slice[s] = TAFloat::NAN;
            } else if self.counts[s] == self.period + 1 {
                let tr = super::trange::trange_inc_raw(h, l, prev_closes_slice[s]);
                tr_sums_slice[s] += tr;
                prev_atrs_slice[s] = tr_sums_slice[s] / self.period as TAFloat;
                prev_closes_slice[s] = c;
                output_slice[s] = prev_atrs_slice[s];
            } else {
                let tr = super::trange::trange_inc_raw(h, l, prev_closes_slice[s]);
                prev_atrs_slice[s] = prev_atrs_slice[s].mul_add((self.period - 1) as TAFloat, tr)
                    / (self.period as TAFloat);
                prev_closes_slice[s] = c;
                output_slice[s] = prev_atrs_slice[s];
            }
        }

        Ok(crate::ta::types::TAArrowArray::new(out_buffer.into(), None))
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{Float64Array, UInt64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("__kand_period", DataType::UInt64, false),
            Field::new("__kand_count", DataType::UInt64, false),
            Field::new("__kand_prev_atr", DataType::Float64, false),
            Field::new("__kand_prev_close", DataType::Float64, false),
            Field::new("__kand_tr_sum", DataType::Float64, false),
        ]));

        let period_arr = Arc::new(UInt64Array::from(vec![
            self.period as u64;
            self.num_streams
        ])) as Arc<dyn arrow::array::Array>;
        let counts_arr = Arc::new(UInt64Array::from(
            self.counts.iter().map(|&c| c as u64).collect::<Vec<_>>(),
        )) as Arc<dyn arrow::array::Array>;

        let prev_atrs_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(self.prev_atrs.as_slice().into(), 0, self.num_streams),
            None,
        )) as Arc<dyn arrow::array::Array>;
        let prev_closes_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(
                self.prev_closes.as_slice().into(),
                0,
                self.num_streams,
            ),
            None,
        )) as Arc<dyn arrow::array::Array>;
        let tr_sums_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(self.tr_sums.as_slice().into(), 0, self.num_streams),
            None,
        )) as Arc<dyn arrow::array::Array>;

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                period_arr,
                counts_arr,
                prev_atrs_arr,
                prev_closes_arr,
                tr_sums_arr,
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
        let prev_atrs = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;
        let prev_closes = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;
        let tr_sums = batch
            .column(4)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;

        self.period = period;
        self.num_streams = num_streams;
        self.counts = counts.values().iter().map(|&c| c as usize).collect();

        self.prev_atrs = arrow_buffer::MutableBuffer::from_len_zeroed(
            prev_atrs.len() * std::mem::size_of::<TAFloat>(),
        );
        self.prev_atrs
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(prev_atrs.values());

        self.prev_closes = arrow_buffer::MutableBuffer::from_len_zeroed(
            prev_closes.len() * std::mem::size_of::<TAFloat>(),
        );
        self.prev_closes
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(prev_closes.values());

        self.tr_sums = arrow_buffer::MutableBuffer::from_len_zeroed(
            tr_sums.len() * std::mem::size_of::<TAFloat>(),
        );
        self.tr_sums
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(tr_sums.values());

        Ok(())
    }
}

/// Calculates Average True Range (ATR) without input validation for high performance.
pub fn atr_raw(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_period: usize,
    output_atr: &mut [TAFloat],
) {
    let len = input_high.len();
    let lookback = opt_period;

    // Calculate first TR values and initial ATR (SMA of TR)
    let mut tr_sum = 0.0;
    let mut prev_close = input_close[0];

    for i in 1..=lookback {
        // Use trange_inc_raw
        let tr = trange::trange_inc_raw(input_high[i], input_low[i], prev_close);
        tr_sum += tr;
        prev_close = input_close[i];
    }
    output_atr[lookback] = tr_sum / (opt_period as TAFloat);

    // Calculate remaining ATR values using RMA
    for i in (lookback + 1)..len {
        let tr = trange::trange_inc_raw(input_high[i], input_low[i], input_close[i - 1]);
        output_atr[i] =
            output_atr[i - 1].mul_add((opt_period - 1) as TAFloat, tr) / (opt_period as TAFloat);
    }
}

/// Calculates Average True Range (ATR) for an entire price series.
///
/// # Description
/// The Average True Range (ATR) is a technical analysis indicator that measures market volatility
/// by decomposing the entire range of an asset price for a given period.
///
/// # Mathematical Formula
/// ```text
/// TR = max(high - low, |high - prev_close|, |low - prev_close|)
/// First ATR = SMA(TR, period)
/// Subsequent ATR = ((period-1) * prev_ATR + TR) / period
/// ```
///
/// # Calculation Steps
/// 1. Calculate True Range (TR) for each period
/// 2. First ATR value is the Simple Moving Average (SMA) of TR over the specified period
/// 3. Subsequent ATR values use Wilder's RMA (Running Moving Average) formula
/// 4. First (period) values will be NaN as they require full period data
///
/// # Arguments
/// * `input_high` - Array of high prices
/// * `input_low` - Array of low prices
/// * `input_close` - Array of close prices
/// * `opt_period` - The time period for ATR calculation (must be >= 2)
/// * `output_atr` - Array to store calculated ATR values
///
/// # Returns
/// * `Result<(), KandError>` - Empty result on success, error otherwise
///
/// # Errors
/// * `KandError::InvalidData` - If input arrays are empty
/// * `KandError::LengthMismatch` - If input arrays have different lengths
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
/// * `KandError::InsufficientData` - If input length <= lookback period
/// * `KandError::NaNDetected` - If any input value is NaN
///
/// # Example
/// ```
/// use kand::ta::ohlcv::atr;
///
/// let input_high = vec![10.0, 12.0, 15.0, 14.0, 13.0];
/// let input_low = vec![8.0, 9.0, 11.0, 10.0, 9.0];
/// let input_close = vec![9.0, 11.0, 14.0, 12.0, 11.0];
/// let opt_period = 3;
/// let mut output_atr = vec![0.0; 5];
///
/// atr::atr(
///     &input_high,
///     &input_low,
///     &input_close,
///     opt_period,
///     &mut output_atr,
/// )
/// .unwrap();
/// ```
pub fn atr(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_period: usize,
    output_atr: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_high.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        // Empty data check
        if len == 0 {
            return Err(KandError::InvalidData);
        }

        // Data sufficiency check
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }

        // Length consistency check
        if len != input_low.len() || len != input_close.len() || len != output_atr.len() {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for i in 0..len {
            // NaN check
            if input_high[i].is_nan() || input_low[i].is_nan() || input_close[i].is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    atr_raw(input_high, input_low, input_close, opt_period, output_atr);

    // Fill initial values with NAN
    output_atr[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Calculates the next ATR value without input validation.
pub fn atr_inc_raw(
    input_high: TAFloat,
    input_low: TAFloat,
    prev_close: TAFloat,
    prev_atr: TAFloat,
    opt_period: usize,
) -> TAFloat {
    let tr = trange::trange_inc_raw(input_high, input_low, prev_close);
    prev_atr.mul_add((opt_period - 1) as TAFloat, tr) / (opt_period as TAFloat)
}

/// Calculates the next ATR value using the previous ATR value and current price data.
///
/// # Description
/// This function provides an efficient way to calculate the next ATR value incrementally,
/// using the previous ATR value and current price data. It uses Wilder's RMA formula.
///
/// # Mathematical Formula
/// ```text
/// TR = max(high - low, |high - prev_close|, |low - prev_close|)
/// ATR = ((prev_ATR * (period-1)) + TR) / period
/// ```
///
/// # Arguments
/// * `input_high` - Current period's high price
/// * `input_low` - Current period's low price
/// * `prev_close` - Previous period's close price
/// * `prev_atr` - Previous period's ATR value
/// * `opt_period` - The time period for ATR calculation (must be >= 2)
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The calculated ATR value on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
/// * `KandError::NaNDetected` - If any input value is NaN
///
/// # Example
/// ```
/// use kand::ta::ohlcv::atr::atr_inc;
///
/// let input_high = 15.0;
/// let input_low = 11.0;
/// let prev_close = 12.0;
/// let prev_atr = 3.0;
/// let opt_period = 3;
///
/// let output_atr = atr_inc(input_high, input_low, prev_close, prev_atr, opt_period).unwrap();
/// ```
pub fn atr_inc(
    input_high: TAFloat,
    input_low: TAFloat,
    prev_close: TAFloat,
    prev_atr: TAFloat,
    opt_period: usize,
) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        // NaN check
        if input_high.is_nan() || input_low.is_nan() || prev_close.is_nan() || prev_atr.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(atr_inc_raw(
        input_high, input_low, prev_close, prev_atr, opt_period,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    atr_arrow,
    crate::ta::ohlcv::atr::atr_raw,
    inputs: { input_high, input_low, input_close },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;

    #[test]
    fn test_stateful_atr() {
        let mut atr_state = StatefulATR::new(3).unwrap();

        // Needs 1 (close) + 3 (lookback) = 4 values
        assert!(atr_state.next((10.0, 8.0, 9.0)).unwrap().is_nan());
        assert!(atr_state.next((12.0, 9.0, 11.0)).unwrap().is_nan());
        assert!(atr_state.next((15.0, 11.0, 14.0)).unwrap().is_nan());
        let val = atr_state.next((14.0, 10.0, 12.0)).unwrap();

        // TRs:
        // t1: max(12-9, |12-9|, |9-9|) = 3.0
        // t2: max(15-11, |15-11|, |11-11|) = 4.0
        // t3: max(14-10, |14-14|, |10-14|) = 4.0
        // sum = 11.0 / 3 = 3.666...
        assert_relative_eq!(val, 3.666_666_666_666_666, epsilon = 0.0001);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_atr() {
        let mut batch_atr = BatchATR::new(3, 2).unwrap();

        // t0
        let h = TAArrowArray::from(vec![10.0, 20.0]);
        let l = TAArrowArray::from(vec![8.0, 18.0]);
        let c = TAArrowArray::from(vec![9.0, 19.0]);
        let out = batch_atr.next_batch((h, l, c)).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let h = TAArrowArray::from(vec![12.0, 22.0]);
        let l = TAArrowArray::from(vec![9.0, 19.0]);
        let c = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_atr.next_batch((h, l, c)).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let h = TAArrowArray::from(vec![15.0, 25.0]);
        let l = TAArrowArray::from(vec![11.0, 21.0]);
        let c = TAArrowArray::from(vec![14.0, 24.0]);
        let out = batch_atr.next_batch((h, l, c)).unwrap();
        assert!(out.value(0).is_nan());

        // t3
        let h = TAArrowArray::from(vec![14.0, 24.0]);
        let l = TAArrowArray::from(vec![10.0, 20.0]);
        let c = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_atr.next_batch((h, l, c)).unwrap();
        assert_relative_eq!(out.value(0), 3.666_666_666_666_666, epsilon = 0.0001);
    }

    #[test]
    fn test_atr_calculation() {
        let input_high = vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35010.8, 35047.1, 35091.4,
        ];
        let input_low = vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35202.8, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34911.0, 34952.5, 34977.9, 35039.0,
        ];
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let opt_period = 14;
        let mut output_atr = vec![0.0; input_high.len()];

        atr(
            &input_high,
            &input_low,
            &input_close,
            opt_period,
            &mut output_atr,
        )
        .unwrap();

        // First 13 values should be NaN
        for value in output_atr.iter().take(13) {
            assert!(value.is_nan());
        }

        // Compare with known values
        let expected_values = [
            63.185_714_285_714_7,
            66.372_448_979_592_43,
            68.602_988_338_192_87,
            67.524_203_456_893_39,
            67.451_046_067_115_29,
            67.118_828_490_892_98,
            69.646_055_027_257_76,
            69.914_193_953_882_3,
            69.084_608_671_462_35,
            69.092_850_909_214_83,
            67.900_504_415_699_59,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_atr[i + 14], *expected, epsilon = 0.0001);
        }

        // Now test incremental calculation matches regular calculation
        let mut prev_atr = output_atr[14]; // First valid ATR value

        // Test each incremental step
        for i in 15..19 {
            let result = atr_inc(
                input_high[i],
                input_low[i],
                input_close[i - 1],
                prev_atr,
                opt_period,
            )
            .unwrap();
            assert_relative_eq!(result, output_atr[i], epsilon = 0.0001);
            prev_atr = result;
        }
    }
}
