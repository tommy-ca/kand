use crate::{KandError, TAFloat};

/// Calculates the lookback period required for RSI (Relative Strength Index) calculation.
///
/// The lookback period equals the input parameter period since RSI needs historical data points
/// to establish the initial average gain and loss values.
///
/// # Arguments
/// * `opt_period` - The number of periods to look back for RSI calculation (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The required lookback period on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
///
/// # Example
/// ```
/// use kand::ta::ohlcv::rsi;
///
/// let opt_period = 14;
/// let lookback = rsi::lookback(opt_period).unwrap();
/// assert_eq!(lookback, 14);
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period)
}

/// Stateful implementation of Relative Strength Index (RSI).
#[derive(Clone)]
pub struct StatefulRSI {
    period: usize,
    count: usize,
    avg_gain: TAFloat,
    avg_loss: TAFloat,
    prev_price: TAFloat,
}

impl StatefulRSI {
    /// Creates a new StatefulRSI instance.
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
            avg_gain: 0.0,
            avg_loss: 0.0,
            prev_price: 0.0,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulRSI {
    type Input = TAFloat;
    type Output = TAFloat;

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        self.count += 1;
        if self.count == 1 {
            self.prev_price = input;
            Ok(TAFloat::NAN)
        } else if self.count <= self.period {
            let diff = input - self.prev_price;
            if diff > 0.0 {
                self.avg_gain += diff;
            } else {
                self.avg_loss += diff.abs();
            }
            self.prev_price = input;

            if self.count == self.period + 1 {
                self.avg_gain /= self.period as TAFloat;
                self.avg_loss /= self.period as TAFloat;

                if self.avg_loss == 0.0 {
                    Ok(100.0)
                } else {
                    let rs = self.avg_gain / self.avg_loss;
                    Ok(100.0 - (100.0 / (1.0 + rs)))
                }
            } else {
                Ok(TAFloat::NAN)
            }
        } else {
            let diff = input - self.prev_price;
            let (curr_gain, curr_loss) = if diff > 0.0 {
                (diff, 0.0)
            } else {
                (0.0, diff.abs())
            };

            let smoothing = self.period as TAFloat;
            self.avg_gain = self.avg_gain.mul_add(smoothing - 1.0, curr_gain) / smoothing;
            self.avg_loss = self.avg_loss.mul_add(smoothing - 1.0, curr_loss) / smoothing;
            self.prev_price = input;

            if self.avg_loss == 0.0 {
                Ok(100.0)
            } else {
                let rs = self.avg_gain / self.avg_loss;
                Ok(100.0 - (100.0 / (1.0 + rs)))
            }
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
            Field::new("__kand_avg_gain", DataType::Float64, false),
            Field::new("__kand_avg_loss", DataType::Float64, false),
            Field::new("__kand_prev_price", DataType::Float64, false),
        ]));

        let period_arr = UInt64Array::from(vec![self.period as u64]);
        let count_arr = UInt64Array::from(vec![self.count as u64]);
        let avg_gain_arr = Float64Array::from(vec![self.avg_gain]);
        let avg_loss_arr = Float64Array::from(vec![self.avg_loss]);
        let prev_price_arr = Float64Array::from(vec![self.prev_price]);

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                Arc::new(period_arr),
                Arc::new(count_arr),
                Arc::new(avg_gain_arr),
                Arc::new(avg_loss_arr),
                Arc::new(prev_price_arr),
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
        let avg_gain = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);
        let avg_loss = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);
        let prev_price = batch
            .column(4)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?
            .value(0);

        self.period = period;
        self.count = count;
        self.avg_gain = avg_gain;
        self.avg_loss = avg_loss;
        self.prev_price = prev_price;

        Ok(())
    }
}

/// Vectorized implementation of Relative Strength Index (RSI) for multiple independent streams.
#[cfg(feature = "arrow")]
pub struct BatchRSI {
    period: usize,
    num_streams: usize,
    counts: Vec<usize>,
    avg_gains: arrow_buffer::MutableBuffer,
    avg_losses: arrow_buffer::MutableBuffer,
    prev_prices: arrow_buffer::MutableBuffer,
}

#[cfg(feature = "arrow")]
impl Clone for BatchRSI {
    fn clone(&self) -> Self {
        let mut new_avg_gains = arrow_buffer::MutableBuffer::new(self.avg_gains.len());
        new_avg_gains.extend_from_slice(self.avg_gains.as_slice());
        let mut new_avg_losses = arrow_buffer::MutableBuffer::new(self.avg_losses.len());
        new_avg_losses.extend_from_slice(self.avg_losses.as_slice());
        let mut new_prev_prices = arrow_buffer::MutableBuffer::new(self.prev_prices.len());
        new_prev_prices.extend_from_slice(self.prev_prices.as_slice());

        Self {
            period: self.period,
            num_streams: self.num_streams,
            counts: self.counts.clone(),
            avg_gains: new_avg_gains,
            avg_losses: new_avg_losses,
            prev_prices: new_prev_prices,
        }
    }
}

#[cfg(feature = "arrow")]
impl BatchRSI {
    /// Creates a new BatchRSI instance.
    pub fn new(period: usize, num_streams: usize) -> Result<Self, KandError> {
        use std::mem::size_of;
        #[cfg(feature = "check")]
        {
            if period < 2 || num_streams == 0 {
                return Err(KandError::InvalidParameter);
            }
        }

        let mut avg_gains = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        avg_gains.resize(num_streams * size_of::<TAFloat>(), 0);

        let mut avg_losses = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        avg_losses.resize(num_streams * size_of::<TAFloat>(), 0);

        let mut prev_prices = arrow_buffer::MutableBuffer::new(num_streams * size_of::<TAFloat>());
        prev_prices.resize(num_streams * size_of::<TAFloat>(), 0);

        Ok(Self {
            period,
            num_streams,
            counts: vec![0; num_streams],
            avg_gains,
            avg_losses,
            prev_prices,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchRSI {
    type Input = crate::ta::types::TAArrowArray;
    type Output = crate::ta::types::TAArrowArray;

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        use std::mem::size_of;

        if input.len() != self.num_streams {
            return Err(KandError::LengthMismatch);
        }

        let input_values = input.values();
        let avg_gains_slice = self.avg_gains.typed_data_mut::<TAFloat>();
        let avg_losses_slice = self.avg_losses.typed_data_mut::<TAFloat>();
        let prev_prices_slice = self.prev_prices.typed_data_mut::<TAFloat>();

        let (ptr, out_buffer) = crate::helper::buffer_pool::create_pooled_buffer(
            self.num_streams * size_of::<TAFloat>(),
        );
        let output_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut TAFloat, self.num_streams) };

        let smoothing = self.period as TAFloat;

        for s in 0..self.num_streams {
            let val = input_values[s];
            self.counts[s] += 1;

            if self.counts[s] == 1 {
                prev_prices_slice[s] = val;
                output_slice[s] = TAFloat::NAN;
            } else if self.counts[s] <= self.period {
                let diff = val - prev_prices_slice[s];
                if diff > 0.0 {
                    avg_gains_slice[s] += diff;
                } else {
                    avg_losses_slice[s] += diff.abs();
                }
                prev_prices_slice[s] = val;

                if self.counts[s] == self.period + 1 {
                    avg_gains_slice[s] /= smoothing;
                    avg_losses_slice[s] /= smoothing;

                    if avg_losses_slice[s] == 0.0 {
                        output_slice[s] = 100.0;
                    } else {
                        let rs = avg_gains_slice[s] / avg_losses_slice[s];
                        output_slice[s] = 100.0 - (100.0 / (1.0 + rs));
                    }
                } else {
                    output_slice[s] = TAFloat::NAN;
                }
            } else {
                let diff = val - prev_prices_slice[s];
                let (curr_gain, curr_loss) = if diff > 0.0 {
                    (diff, 0.0)
                } else {
                    (0.0, diff.abs())
                };

                avg_gains_slice[s] =
                    avg_gains_slice[s].mul_add(smoothing - 1.0, curr_gain) / smoothing;
                avg_losses_slice[s] =
                    avg_losses_slice[s].mul_add(smoothing - 1.0, curr_loss) / smoothing;
                prev_prices_slice[s] = val;

                if avg_losses_slice[s] == 0.0 {
                    output_slice[s] = 100.0;
                } else {
                    let rs = avg_gains_slice[s] / avg_losses_slice[s];
                    output_slice[s] = 100.0 - (100.0 / (1.0 + rs));
                }
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
            Field::new("__kand_avg_gain", DataType::Float64, false),
            Field::new("__kand_avg_loss", DataType::Float64, false),
            Field::new("__kand_prev_price", DataType::Float64, false),
        ]));

        let period_arr = Arc::new(UInt64Array::from(vec![
            self.period as u64;
            self.num_streams
        ])) as Arc<dyn arrow::array::Array>;
        let counts_arr = Arc::new(UInt64Array::from(
            self.counts.iter().map(|&c| c as u64).collect::<Vec<_>>(),
        )) as Arc<dyn arrow::array::Array>;

        let avg_gains_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(self.avg_gains.as_slice().into(), 0, self.num_streams),
            None,
        )) as Arc<dyn arrow::array::Array>;
        let avg_losses_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(self.avg_losses.as_slice().into(), 0, self.num_streams),
            None,
        )) as Arc<dyn arrow::array::Array>;
        let prev_prices_arr = Arc::new(Float64Array::new(
            arrow_buffer::ScalarBuffer::new(
                self.prev_prices.as_slice().into(),
                0,
                self.num_streams,
            ),
            None,
        )) as Arc<dyn arrow::array::Array>;

        arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                period_arr,
                counts_arr,
                avg_gains_arr,
                avg_losses_arr,
                prev_prices_arr,
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
        let avg_gains = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;
        let avg_losses = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;
        let prev_prices = batch
            .column(4)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(KandError::InvalidData)?;

        self.period = period;
        self.num_streams = num_streams;
        self.counts = counts.values().iter().map(|&c| c as usize).collect();

        self.avg_gains = arrow_buffer::MutableBuffer::from_len_zeroed(
            avg_gains.len() * std::mem::size_of::<TAFloat>(),
        );
        self.avg_gains
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(avg_gains.values());

        self.avg_losses = arrow_buffer::MutableBuffer::from_len_zeroed(
            avg_losses.len() * std::mem::size_of::<TAFloat>(),
        );
        self.avg_losses
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(avg_losses.values());

        self.prev_prices = arrow_buffer::MutableBuffer::from_len_zeroed(
            prev_prices.len() * std::mem::size_of::<TAFloat>(),
        );
        self.prev_prices
            .typed_data_mut::<TAFloat>()
            .copy_from_slice(prev_prices.values());

        Ok(())
    }
}

/// Calculates RSI without input validation for high performance.
pub fn rsi_raw(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_rsi: &mut [TAFloat],
    output_avg_gain: &mut [TAFloat],
    output_avg_loss: &mut [TAFloat],
) {
    let len = input_prices.len();
    let lookback = opt_period;

    let mut gains = 0.0;
    let mut losses = 0.0;

    // Calculate initial gains/losses sum
    for i in 1..=lookback {
        let diff = input_prices[i] - input_prices[i - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses += diff.abs();
        }
    }

    // Calculate first RSI value
    let first_avg_gain = gains / opt_period as TAFloat;
    let first_avg_loss = losses / opt_period as TAFloat;

    output_avg_gain[lookback] = first_avg_gain;
    output_avg_loss[lookback] = first_avg_loss;

    if first_avg_loss == 0.0 {
        output_rsi[lookback] = 100.0;
    } else {
        let rs = first_avg_gain / first_avg_loss;
        output_rsi[lookback] = 100.0 - (100.0 / (1.0 + rs));
    }

    // Calculate remaining RSI values using smoothed averages
    let mut prev_avg_gain = first_avg_gain;
    let mut prev_avg_loss = first_avg_loss;
    let smoothing = opt_period as TAFloat;

    for i in lookback + 1..len {
        let diff = input_prices[i] - input_prices[i - 1];
        let (curr_gain, curr_loss) = if diff > 0.0 {
            (diff, 0.0)
        } else {
            (0.0, diff.abs())
        };

        let curr_avg_gain = prev_avg_gain.mul_add(smoothing - 1.0, curr_gain) / smoothing;
        let curr_avg_loss = prev_avg_loss.mul_add(smoothing - 1.0, curr_loss) / smoothing;

        output_avg_gain[i] = curr_avg_gain;
        output_avg_loss[i] = curr_avg_loss;

        if curr_avg_loss == 0.0 {
            output_rsi[i] = 100.0;
        } else {
            let rs = curr_avg_gain / curr_avg_loss;
            output_rsi[i] = 100.0 - (100.0 / (1.0 + rs));
        }

        prev_avg_gain = curr_avg_gain;
        prev_avg_loss = curr_avg_loss;
    }
}

/// Calculates Relative Strength Index (RSI) for a price series.
///
/// RSI is a momentum oscillator that measures the speed and magnitude of recent price changes
/// to evaluate overbought or oversold conditions. It oscillates between 0 and 100, with
/// values above 70 generally indicating overbought conditions and values below 30 indicating
/// oversold conditions.
///
/// # Mathematical Formula
/// ```text
/// RSI = 100 - (100 / (1 + RS))
/// where:
/// RS = Average Gain / Average Loss
///
/// Initial Average Gain = Sum of Gains over past n periods / n
/// Initial Average Loss = Sum of Losses over past n periods / n
///
/// Subsequent values:
/// Average Gain = ((Previous Average Gain) × (n-1) + Current Gain) / n
/// Average Loss = ((Previous Average Loss) × (n-1) + Current Loss) / n
/// ```
///
/// # Calculation Principle
/// 1. Calculate price changes between consecutive periods
/// 2. Separate gains (positive changes) from losses (negative changes)
/// 3. Calculate initial average gain and loss over first n periods
/// 4. Apply Wilder's smoothing formula for subsequent periods
/// 5. Calculate RS ratio and convert to RSI value
///
/// # Arguments
/// * `input_prices` - Array of price values (typically closing prices)
/// * `opt_period` - The time period for RSI calculation (typical values: 14, 9, or 25)
/// * `output_rsi` - Array to store calculated RSI values
/// * `output_avg_gain` - Array to store average gain values for each period
/// * `output_avg_loss` - Array to store average loss values for each period
///
/// # Returns
/// * `Result<(), KandError>` - Ok(()) on successful calculation
///
/// # Errors
/// * `KandError::InvalidData` - If input array is empty
/// * `KandError::LengthMismatch` - If input and output arrays have different lengths
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
/// * `KandError::InsufficientData` - If input length is less than or equal to lookback period
/// * `KandError::NaNDetected` - If any input value is NaN (when "`check-nan`" feature is enabled)
///
/// # Example
/// ```
/// use kand::ta::ohlcv::rsi;
///
/// let input_prices = vec![44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42];
/// let opt_period = 5;
/// let mut output_rsi = vec![0.0; input_prices.len()];
/// let mut output_avg_gain = vec![0.0; input_prices.len()];
/// let mut output_avg_loss = vec![0.0; input_prices.len()];
///
/// rsi::rsi(
///     &input_prices,
///     opt_period,
///     &mut output_rsi,
///     &mut output_avg_gain,
///     &mut output_avg_loss,
/// )
/// .unwrap();
/// ```
pub fn rsi(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_rsi: &mut [TAFloat],
    output_avg_gain: &mut [TAFloat],
    output_avg_loss: &mut [TAFloat],
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
        if output_rsi.len() != len || output_avg_gain.len() != len || output_avg_loss.len() != len {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_prices {
            // NaN check
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    rsi_raw(
        input_prices,
        opt_period,
        output_rsi,
        output_avg_gain,
        output_avg_loss,
    );

    // Fill initial values with NAN
    output_rsi[..lookback].fill(TAFloat::NAN);
    output_avg_gain[..lookback].fill(TAFloat::NAN);
    output_avg_loss[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Calculates the latest RSI value incrementally without validation.
pub fn rsi_inc_raw(
    input_curr_price: TAFloat,
    prev_price: TAFloat,
    prev_avg_gain: TAFloat,
    prev_avg_loss: TAFloat,
    opt_period: usize,
) -> (TAFloat, TAFloat, TAFloat) {
    let diff = input_curr_price - prev_price;
    let (curr_gain, curr_loss) = if diff > 0.0 {
        (diff, 0.0)
    } else {
        (0.0, diff.abs())
    };

    let smoothing = opt_period as TAFloat;
    let output_avg_gain = prev_avg_gain.mul_add(smoothing - 1.0, curr_gain) / smoothing;
    let output_avg_loss = prev_avg_loss.mul_add(smoothing - 1.0, curr_loss) / smoothing;

    let output_rsi = if output_avg_loss == 0.0 {
        100.0
    } else {
        let rs = output_avg_gain / output_avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    };

    (output_rsi, output_avg_gain, output_avg_loss)
}

/// Calculates the latest RSI value incrementally using previous average gain and loss values.
///
/// This function optimizes RSI calculation for real-time data by using the previous period's
/// average gain and loss values to calculate the current RSI value, without needing the entire
/// price history.
///
/// # Mathematical Formula
/// ```text
/// Average Gain = ((Previous Average Gain) × (n-1) + Current Gain) / n
/// Average Loss = ((Previous Average Loss) × (n-1) + Current Loss) / n
/// RS = Average Gain / Average Loss
/// RSI = 100 - (100 / (1 + RS))
/// ```
///
/// # Arguments
/// * `input_curr_price` - Current period's price value
/// * `prev_price` - Previous period's price value
/// * `prev_avg_gain` - Previous period's average gain
/// * `prev_avg_loss` - Previous period's average loss
/// * `opt_period` - The time period for RSI calculation
///
/// # Returns
/// * `Result<(TAFloat, TAFloat, TAFloat), KandError>` - Tuple containing (RSI value, new average gain, new average loss)
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
/// * `KandError::NaNDetected` - If any input value is NaN (when "`check-nan`" feature is enabled)
///
/// # Example
/// ```
/// use kand::ta::ohlcv::rsi;
///
/// let (rsi_value, avg_gain, avg_loss) = rsi::rsi_inc(
///     45.42, // current price
///     45.10, // previous price
///     0.24,  // previous average gain
///     0.14,  // previous average loss
///     14,    // period
/// )
/// .unwrap();
/// ```
pub fn rsi_inc(
    input_curr_price: TAFloat,
    prev_price: TAFloat,
    prev_avg_gain: TAFloat,
    prev_avg_loss: TAFloat,
    opt_period: usize,
) -> Result<(TAFloat, TAFloat, TAFloat), KandError> {
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
        if input_curr_price.is_nan()
            || prev_price.is_nan()
            || prev_avg_gain.is_nan()
            || prev_avg_loss.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(rsi_inc_raw(
        input_curr_price,
        prev_price,
        prev_avg_gain,
        prev_avg_loss,
        opt_period,
    ))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper_multi!(
    rsi_arrow,
    crate::ta::ohlcv::rsi::rsi_raw,
    inputs: { input_prices },
    params: { opt_period: usize },
    lookback_params: { opt_period },
    outputs: { output_rsi: crate::TAFloat, output_avg_gain: crate::TAFloat, output_avg_loss: crate::TAFloat },
    return_type: { crate::ta::types::TAArrowArray, crate::ta::types::TAArrowArray, crate::ta::types::TAArrowArray }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;

    // Basic functionality tests
    #[test]
    fn test_stateful_rsi() {
        let mut rsi_state = StatefulRSI::new(3).unwrap();

        // Need 1 (price) + 3 (lookback) = 4 values for first valid
        assert!(rsi_state.next(10.0).unwrap().is_nan());
        assert!(rsi_state.next(11.0).unwrap().is_nan());
        assert!(rsi_state.next(12.0).unwrap().is_nan());
        let val = rsi_state.next(13.0).unwrap();
        // Gains: 1, 1, 1 -> avg = 1.0
        // Losses: 0, 0, 0 -> avg = 0.0
        assert_relative_eq!(val, 100.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_rsi() {
        let mut batch_rsi = BatchRSI::new(3, 2).unwrap();

        // t0
        let input = TAArrowArray::from(vec![10.0, 20.0]);
        let out = batch_rsi.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_rsi.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_rsi.next_batch(input).unwrap();
        assert!(out.value(0).is_nan());

        // t3 - first valid
        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let out = batch_rsi.next_batch(input).unwrap();
        assert_relative_eq!(out.value(0), 100.0);
        assert_relative_eq!(out.value(1), 100.0);
    }

    #[test]
    fn test_rsi_calculation() {
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0,
        ];
        let opt_period = 14;
        let mut output_rsi = vec![0.0; input_prices.len()];
        let mut output_avg_gain = vec![0.0; input_prices.len()];
        let mut output_avg_loss = vec![0.0; input_prices.len()];

        rsi(
            &input_prices,
            opt_period,
            &mut output_rsi,
            &mut output_avg_gain,
            &mut output_avg_loss,
        )
        .unwrap();

        // Verify first 14 values are NaN
        for value in output_rsi.iter().take(opt_period) {
            assert!(value.is_nan());
        }

        // Verify against known values
        assert_relative_eq!(output_rsi[14], 37.748_344_370_861_39, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[15], 34.223_538_361_225_86, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[16], 31.518_806_080_459_882, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[17], 33.425_568_632_418_2, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[18], 40.465_006_259_629_995, epsilon = 0.00001);

        // Now test incremental calculation matches regular calculation
        let mut prev_avg_gain = output_avg_gain[opt_period];
        let mut prev_avg_loss = output_avg_loss[opt_period];
        let mut prev_price = input_prices[opt_period];

        // Test each incremental step
        for i in opt_period + 1..input_prices.len() {
            let (result, new_avg_gain, new_avg_loss) = rsi_inc(
                input_prices[i],
                prev_price,
                prev_avg_gain,
                prev_avg_loss,
                opt_period,
            )
            .unwrap();

            assert_relative_eq!(result, output_rsi[i], epsilon = 0.00001);
            assert_relative_eq!(new_avg_gain, output_avg_gain[i], epsilon = 0.00001);
            assert_relative_eq!(new_avg_loss, output_avg_loss[i], epsilon = 0.00001);

            prev_avg_gain = new_avg_gain;
            prev_avg_loss = new_avg_loss;
            prev_price = input_prices[i];
        }
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_rsi_arrow() {
        use crate::ta::types::TAArrowArray;
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0,
        ];
        let input_prices_arrow = TAArrowArray::from(input_prices);
        let opt_period = 14;

        let (output_rsi_arrow, _output_avg_gain_arrow, _output_avg_loss_arrow) =
            rsi_arrow(&input_prices_arrow, opt_period).unwrap();

        // Verify against known values
        assert_relative_eq!(
            output_rsi_arrow.value(14),
            37.748_344_370_861_39,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            output_rsi_arrow.value(15),
            34.223_538_361_225_86,
            epsilon = 0.00001
        );
    }
}
