use crate::{KandError, TAFloat, TAPeriod, helper::period_to_k};

/// Returns the lookback period for EMA without input validation.
#[inline]
#[must_use]
pub const fn lookback_raw(opt_period: TAPeriod) -> TAPeriod {
    opt_period - 1
}

/// Returns the lookback period required for EMA calculation.
///
/// # Description
/// Calculates the minimum number of historical data points needed before generating the first valid EMA value.
/// For EMA, this equals period - 1 since the first value requires a complete period for SMA calculation.
///
/// # Arguments
/// * `opt_period` - The time period for EMA calculation. Must be >= 2.
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period on success, or error on failure.
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2.
///
/// # Example
/// ```
/// use kand::ohlcv::ema;
/// let period = 14;
/// let lookback = ema::lookback(period).unwrap();
/// assert_eq!(lookback, 13); // lookback is period - 1
/// ```
pub const fn lookback(opt_period: TAPeriod) -> Result<TAPeriod, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(lookback_raw(opt_period))
}

/// Stateful implementation of Exponential Moving Average (EMA).
pub struct StatefulEMA {
    period: TAPeriod,
    multiplier: TAFloat,
    prev_ema: TAFloat,
    sum: TAFloat,
    count: usize,
}

impl StatefulEMA {
    /// Creates a new StatefulEMA instance.
    pub fn new(period: TAPeriod, opt_k: Option<TAFloat>) -> Result<Self, KandError> {
        #[cfg(feature = "check")]
        {
            if period < 2 {
                return Err(KandError::InvalidParameter);
            }
        }
        let multiplier = match opt_k {
            Some(k) => k,
            None => 2.0 / (period as TAFloat + 1.0),
        };
        Ok(Self {
            period,
            multiplier,
            prev_ema: 0.0,
            sum: 0.0,
            count: 0,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulEMA {
    type Input = TAFloat;
    type Output = TAFloat;

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        self.count += 1;
        if self.count < self.period {
            self.sum += input;
            Ok(TAFloat::NAN)
        } else if self.count == self.period {
            self.sum += input;
            self.prev_ema = self.sum / self.period as TAFloat;
            Ok(self.prev_ema)
        } else {
            self.prev_ema = (input - self.prev_ema).mul_add(self.multiplier, self.prev_ema);
            Ok(self.prev_ema)
        }
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{UInt64Array, Float64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("period", DataType::UInt64, false),
            Field::new("multiplier", DataType::Float64, false),
            Field::new("prev_ema", DataType::Float64, false),
            Field::new("sum", DataType::Float64, false),
            Field::new("count", DataType::UInt64, false),
        ]));

        let period_arr = UInt64Array::from(vec![self.period as u64]);
        let multiplier_arr = Float64Array::from(vec![self.multiplier]);
        let prev_ema_arr = Float64Array::from(vec![self.prev_ema]);
        let sum_arr = Float64Array::from(vec![self.sum]);
        let count_arr = UInt64Array::from(vec![self.count as u64]);

        arrow::record_batch::RecordBatch::try_new(schema, vec![
            Arc::new(period_arr),
            Arc::new(multiplier_arr),
            Arc::new(prev_ema_arr),
            Arc::new(sum_arr),
            Arc::new(count_arr),
        ]).map_err(|_| KandError::InvalidData)
    }

    #[cfg(feature = "arrow")]
    fn from_record_batch(&mut self, batch: &arrow::record_batch::RecordBatch) -> Result<(), KandError> {
        use arrow::array::{UInt64Array, Float64Array};

        let period = batch.column(0).as_any().downcast_ref::<UInt64Array>().ok_or(KandError::InvalidData)?.value(0) as usize;
        let multiplier = batch.column(1).as_any().downcast_ref::<Float64Array>().ok_or(KandError::InvalidData)?.value(0);
        let prev_ema = batch.column(2).as_any().downcast_ref::<Float64Array>().ok_or(KandError::InvalidData)?.value(0);
        let sum = batch.column(3).as_any().downcast_ref::<Float64Array>().ok_or(KandError::InvalidData)?.value(0);
        let count = batch.column(4).as_any().downcast_ref::<UInt64Array>().ok_or(KandError::InvalidData)?.value(0) as usize;

        self.period = period;
        self.multiplier = multiplier;
        self.prev_ema = prev_ema;
        self.sum = sum;
        self.count = count;

        Ok(())
    }
}

/// Vectorized implementation of Exponential Moving Average (EMA) for multiple independent streams.
#[cfg(feature = "arrow")]
pub struct BatchEMA {
    period: TAPeriod,
    num_streams: usize,
    multiplier: TAFloat,
    // Buffer storing current sum (for initial SMA) or previous EMA
    states: arrow_buffer::MutableBuffer,
    // Buffer storing whether we are in the initial SMA phase or EMA phase
    counts: Vec<usize>, // Could be optimized to a single usize if all streams sync
}

#[cfg(feature = "arrow")]
impl BatchEMA {
    /// Creates a new BatchEMA instance.
    pub fn new(period: TAPeriod, num_streams: usize, opt_k: Option<TAFloat>) -> Result<Self, KandError> {
        use std::mem::size_of;
        #[cfg(feature = "check")]
        {
            if period < 2 || num_streams == 0 {
                return Err(KandError::InvalidParameter);
            }
        }
        let multiplier = match opt_k {
            Some(k) => k,
            None => 2.0 / (period as TAFloat + 1.0),
        };

        let mut states = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        states.resize(num_streams * size_of::<TAFloat>(), 0);

        Ok(Self {
            period,
            num_streams,
            multiplier,
            states,
            counts: vec![0; num_streams],
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchEMA {
    type Input = crate::ta::types::TAArrowArray;
    type Output = crate::ta::types::TAArrowArray;

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        use arrow::array::Array;
        use std::mem::size_of;

        if input.len() != self.num_streams {
            return Err(KandError::LengthMismatch);
        }

        let input_values = input.values();
        let states_slice = self.states.typed_data_mut::<TAFloat>();
        
        let (ptr, out_buffer) = crate::helper::buffer_pool::create_pooled_buffer(self.num_streams * size_of::<TAFloat>());
        let output_slice = unsafe {
            std::slice::from_raw_parts_mut(ptr as *mut TAFloat, self.num_streams)
        };

        for s in 0..self.num_streams {
            let val = input_values[s];
            self.counts[s] += 1;
            
            if self.counts[s] < self.period {
                states_slice[s] += val;
                output_slice[s] = TAFloat::NAN;
            } else if self.counts[s] == self.period {
                states_slice[s] += val;
                let initial_ema = states_slice[s] / self.period as TAFloat;
                states_slice[s] = initial_ema;
                output_slice[s] = initial_ema;
            } else {
                let prev_ema = states_slice[s];
                let new_ema = (val - prev_ema).mul_add(self.multiplier, prev_ema);
                states_slice[s] = new_ema;
                output_slice[s] = new_ema;
            }
        }

        Ok(crate::ta::types::TAArrowArray::new(out_buffer.into(), None))
    }

    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use arrow::array::{UInt64Array, Float64Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let schema = Arc::new(Schema::new_with_metadata(
            vec![
                Field::new("state", DataType::Float64, false),
                Field::new("count", DataType::UInt64, false),
            ],
            std::collections::HashMap::from([
                ("period".to_string(), self.period.to_string()),
                ("multiplier".to_string(), self.multiplier.to_string()),
            ])
        ));

        let states_arr = Arc::new(Float64Array::new(arrow_buffer::ScalarBuffer::new(self.states.as_slice().into(), 0, self.num_streams), None)) as Arc<dyn arrow::array::Array>;
        let counts_arr = Arc::new(UInt64Array::from(self.counts.iter().map(|&c| c as u64).collect::<Vec<_>>())) as Arc<dyn arrow::array::Array>;

        arrow::record_batch::RecordBatch::try_new(schema, vec![
            states_arr,
            counts_arr,
        ]).map_err(|_| KandError::InvalidData)
    }

    fn from_record_batch(&mut self, batch: &arrow::record_batch::RecordBatch) -> Result<(), KandError> {
        use arrow::array::{UInt64Array, Float64Array};

        let period = batch.schema().metadata().get("period").ok_or(KandError::InvalidData)?.parse().map_err(|_| KandError::InvalidData)?;
        let multiplier = batch.schema().metadata().get("multiplier").ok_or(KandError::InvalidData)?.parse().map_err(|_| KandError::InvalidData)?;
        
        let num_streams = batch.num_rows();
        
        let states = batch.column(0).as_any().downcast_ref::<Float64Array>().ok_or(KandError::InvalidData)?;
        let counts = batch.column(1).as_any().downcast_ref::<UInt64Array>().ok_or(KandError::InvalidData)?;

        self.period = period;
        self.num_streams = num_streams;
        self.multiplier = multiplier;
        
        self.states = arrow_buffer::MutableBuffer::from_len_zeroed(states.len() * std::mem::size_of::<TAFloat>());
        self.states.typed_data_mut::<TAFloat>().copy_from_slice(states.values());
        
        self.counts = counts.values().iter().map(|&c| c as usize).collect();

        Ok(())
    }
}

/// Computes EMA without input validation for high performance.
pub fn ema_raw(
    input_prices: &[TAFloat],
    opt_period: TAPeriod,
    opt_k: Option<TAFloat>,
    output_ema: &mut [TAFloat],
) {
    let len = input_prices.len();
    let lookback = lookback_raw(opt_period);

    // Calculate initial SMA
    let mut sum = input_prices[0];
    for value in input_prices.iter().take(opt_period).skip(1) {
        sum += *value;
    }
    let mut prev_ma = sum / (opt_period as TAFloat);
    output_ema[lookback] = prev_ma;

    // Get multiplier - either custom or default
    let multiplier = match opt_k {
        Some(k) => k,
        None => 2.0 / (opt_period + 1) as TAFloat,
    };

    // Calculate EMA
    for i in opt_period..len {
        prev_ma = (input_prices[i] - prev_ma).mul_add(multiplier, prev_ma);
        output_ema[i] = prev_ma;
    }
}

/// Calculates Exponential Moving Average (EMA) for a price series.
///
/// # Description
/// EMA is a type of moving average that places a greater weight and significance on the most recent data points.
/// The weighting given to each data point decreases exponentially with time.
///
/// # Mathematical Formula
/// ```text
/// Initial EMA = SMA(first n prices)
/// EMA = Price * k + EMA(previous) * (1 - k)
/// where:
/// k = smoothing factor (default is 2/(period+1))
/// ```
///
/// # Calculation Steps
/// 1. Calculate Simple Moving Average (SMA) for initial period
/// 2. Apply EMA formula using smoothing factor k for remaining periods
/// 3. Fill initial values before lookback period with NaN
///
/// # Arguments
/// * `input_prices` - Array of price values to calculate EMA
/// * `opt_period` - The time period for EMA calculation (must be >= 2)
/// * `opt_k` - Optional custom smoothing factor. If None, uses 2/(period+1)
/// * `output_ema` - Array to store calculated EMA values. Must match input length
///
/// # Returns
/// * `Result<(), KandError>` - Unit on success, or error on failure
///
/// # Errors
/// * `KandError::InvalidData` - If input array is empty
/// * `KandError::LengthMismatch` - If output length doesn't match input
/// * `KandError::InvalidParameter` - If period < 2
/// * `KandError::InsufficientData` - If input length < period
/// * `KandError::NaNDetected` - If any input price is NaN (with "`check-nan`")
///
/// # Example
/// ```
/// use kand::ohlcv::ema;
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let period = 3;
/// let mut ema_values = vec![0.0; prices.len()];
///
/// // Calculate EMA with default smoothing
/// ema::ema(&prices, period, None, &mut ema_values).unwrap();
/// ```
pub fn ema(
    input_prices: &[TAFloat],
    opt_period: TAPeriod,
    opt_k: Option<TAFloat>,
    output_ema: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_prices.len();
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
        if output_ema.len() != len {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_prices {
            // NaN check
            if price.is_null() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    ema_raw(input_prices, opt_period, opt_k, output_ema);

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for value in output_ema.iter_mut().take(lookback) {
            *value = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Computes the next EMA value incrementally without input validation.
#[inline]
#[must_use]
pub fn ema_inc_raw(
    input_price: TAFloat,
    prev_ema: TAFloat,
    multiplier: TAFloat,
) -> TAFloat {
    (input_price - prev_ema).mul_add(multiplier, prev_ema)
}

/// Calculates a single EMA value incrementally using the previous EMA.
///
/// # Description
/// Provides an efficient way to update EMA calculations when new data arrives, without reprocessing
/// the entire dataset. Uses the previous EMA value and current price to compute the new EMA.
///
/// # Mathematical Formula
/// ```text
/// EMA = Price * k + EMA(previous) * (1 - k)
/// where:
/// k = smoothing factor (default is 2/(period+1))
/// ```
///
/// # Arguments
/// * `input_price` - The current period's price value
/// * `prev_ema` - The previous period's EMA value
/// * `opt_period` - The time period for EMA calculation (must be >= 2)
/// * `opt_k` - Optional custom smoothing factor. If None, uses 2/(period+1)
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The new EMA value on success, or error on failure
///
/// # Errors
/// * `KandError::InvalidParameter` - If period < 2
/// * `KandError::NaNDetected` - If price or previous EMA is NaN (with "`check-nan`")
///
/// # Example
/// ```
/// use kand::ohlcv::ema;
/// let current_price = 15.0;
/// let prev_ema = 14.5;
/// let period = 14;
///
/// // Calculate next EMA with default smoothing
/// let new_ema = ema::ema_inc(current_price, prev_ema, period, None).unwrap();
/// ```
pub fn ema_inc(
    input_price: TAFloat,
    prev_ema: TAFloat,
    opt_period: TAPeriod,
    opt_k: Option<TAFloat>,
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
        if input_price.is_null() || prev_ema.is_null() {
            return Err(KandError::NaNDetected);
        }
    }

    let multiplier = match opt_k {
        Some(k) => k,
        None => period_to_k(opt_period)?,
    };
    Ok(ema_inc_raw(input_price, prev_ema, multiplier))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    ema_arrow,
    crate::ta::ohlcv::ema::ema_raw,
    inputs: { input_prices },
    params: { opt_period: TAPeriod, opt_k: Option<TAFloat> },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use arrow::array::Array;
    use approx::assert_relative_eq;

    use super::*;

    // Basic functionality tests
    #[test]
    fn test_stateful_ema() {
        use crate::ta::traits::Indicator;
        let mut ema = StatefulEMA::new(3, None).unwrap();
        assert!(ema.next(10.0).unwrap().is_nan());
        assert!(ema.next(11.0).unwrap().is_nan());
        assert_relative_eq!(ema.next(12.0).unwrap(), 11.0);
        assert_relative_eq!(ema.next(13.0).unwrap(), 12.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_ema() {
        use crate::ta::traits::BatchIndicator;
        use crate::ta::types::TAArrowArray;
        let mut batch_ema = BatchEMA::new(3, 2, None).unwrap();
        
        // t0
        let input = TAArrowArray::from(vec![10.0, 20.0]);
        let out = batch_ema.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t1
        let input = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_ema.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t2 - first valid (SMA)
        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_ema.next_batch(input).unwrap();
        assert_relative_eq!(out.value(0), 11.0);
        assert_relative_eq!(out.value(1), 21.0);

        // t3 - EMA update
        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let out = batch_ema.next_batch(input).unwrap();
        // k = 2/(3+1) = 0.5
        // ema = (13 - 11) * 0.5 + 11 = 12.0
        assert_relative_eq!(out.value(0), 12.0);
        assert_relative_eq!(out.value(1), 22.0);
    }

    #[test]
    fn test_ema_calculation() {
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
        ];
        let opt_period = 14;
        let mut output_ema = vec![0.0; input_prices.len()];

        ema(&input_prices, opt_period, None, &mut output_ema).unwrap();

        // First 13 values should be NaN
        #[cfg(feature = "allow-nan")]
        for value in output_ema.iter().take(13) {
            assert!(value.is_nan());
        }

        // Test first valid value
        let expected_values = [
            35_203.535_714_285_72,
            35_188.437_619_047_625,
            35_168.805_936_507_94,
            35_146.205_144_973_545,
            35_128.497_792_310_41,
            35_120.564_753_335_69,
            35_107.769_452_890_934,
            35_085.333_525_838_81,
            35_067.635_722_393_636,
            35_058.617_626_074_48,
            35_056.375_275_931_22,
            35_059.525_239_140_39,
            35_066.855_207_255_01,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_ema[i + 13], *expected, epsilon = 0.00001);
        }

        let opt_period = 14;
        let mut output_ema = vec![0.0; input_prices.len()];

        ema(&input_prices, opt_period, None, &mut output_ema).unwrap();

        // Now test incremental calculation matches regular calculation
        let mut prev_ema = output_ema[13]; // First valid EMA value

        // Test each incremental step
        for i in 14..18 {
            let result = ema_inc(input_prices[i], prev_ema, opt_period, None).unwrap();
            assert_relative_eq!(result, output_ema[i], epsilon = 0.00001);
            prev_ema = result;
        }
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_ema_arrow() {
        use crate::ta::types::TAArrowArray;
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
        ];
        let input_prices_arrow = TAArrowArray::from(input_prices);
        let opt_period = 14;

        let output_ema_arrow = ema_arrow(&input_prices_arrow, opt_period, None).unwrap();

        // Test first valid value
        let expected_values = [
            35_203.535_714_285_72,
            35_188.437_619_047_625,
            35_168.805_936_507_94,
            35_146.205_144_973_545,
            35_128.497_792_310_41,
            35_120.564_753_335_69,
            35_107.769_452_890_934,
            35_085.333_525_838_81,
            35_067.635_722_393_636,
            35_058.617_626_074_48,
            35_056.375_275_931_22,
            35_059.525_239_140_39,
            35_066.855_207_255_01,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_ema_arrow.value(i + 13), *expected, epsilon = 0.00001);
        }
    }
}
