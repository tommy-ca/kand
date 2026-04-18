use super::ema;
use crate::{KandError, TAFloat};

#[cfg(feature = "arrow")]
use crate::ta::types::TAArrowArray;

/// Calculate the lookback period required for MACD calculation
///
/// Returns the minimum number of data points needed before the first valid MACD result.
pub const fn lookback(
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_signal_period: usize,
) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_fast_period < 2 || opt_slow_period < 2 || opt_signal_period < 2 {
            return Err(KandError::InvalidParameter);
        }
        if opt_fast_period >= opt_slow_period {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_slow_period + opt_signal_period - 2)
}

/// Stateful implementation of MACD.
#[derive(Clone)]
pub struct StatefulMACD {
    fast_ema: ema::StatefulEMA,
    slow_ema: ema::StatefulEMA,
    signal_ema: ema::StatefulEMA,
}

impl StatefulMACD {
    /// Creates a new StatefulMACD instance.
    pub fn new(
        opt_fast_period: usize,
        opt_slow_period: usize,
        opt_signal_period: usize,
    ) -> Result<Self, KandError> {
        let fast_ema = ema::StatefulEMA::new_ext(opt_fast_period, None)?;
        let slow_ema = ema::StatefulEMA::new_ext(opt_slow_period, None)?;
        let signal_ema = ema::StatefulEMA::new_ext(opt_signal_period, None)?;
        let _ = lookback(opt_fast_period, opt_slow_period, opt_signal_period)?;

        Ok(Self {
            fast_ema,
            slow_ema,
            signal_ema,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulMACD {
    type Input = TAFloat;
    type Output = (TAFloat, TAFloat, TAFloat);

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        // Atomic "Transactional" Update Pattern
        let mut fast_clone = self.fast_ema.clone();
        let mut slow_clone = self.slow_ema.clone();
        let mut signal_clone = self.signal_ema.clone();

        let fast = fast_clone.next((input,))?;
        let slow = slow_clone.next((input,))?;

        if fast.is_nan() || slow.is_nan() {
            // Still warming up
            self.fast_ema = fast_clone;
            self.slow_ema = slow_clone;
            return Ok((TAFloat::NAN, TAFloat::NAN, TAFloat::NAN));
        }

        let macd_val = fast - slow;
        let signal = signal_clone.next((macd_val,))?;

        // Success: Commit clones
        self.fast_ema = fast_clone;
        self.slow_ema = slow_clone;
        self.signal_ema = signal_clone;

        if signal.is_nan() {
            Ok((TAFloat::NAN, TAFloat::NAN, TAFloat::NAN))
        } else {
            Ok((macd_val, signal, macd_val - signal))
        }
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        use crate::ta::traits::Indicator;
        // MACD state is the composition of its EMAs
        let fast_batch = self.fast_ema.to_record_batch()?;
        let slow_batch = self.slow_ema.to_record_batch()?;
        let signal_batch = self.signal_ema.to_record_batch()?;

        // Interleave columns with prefixes
        let mut fields = Vec::new();
        let mut columns = Vec::new();

        for (f, c) in fast_batch
            .schema()
            .fields()
            .iter()
            .zip(fast_batch.columns().iter())
        {
            fields.push(arrow::datatypes::Field::new(
                format!("fast_{}", f.name()),
                f.data_type().clone(),
                f.is_nullable(),
            ));
            columns.push(c.clone());
        }

        for (f, c) in slow_batch
            .schema()
            .fields()
            .iter()
            .zip(slow_batch.columns().iter())
        {
            fields.push(arrow::datatypes::Field::new(
                format!("slow_{}", f.name()),
                f.data_type().clone(),
                f.is_nullable(),
            ));
            columns.push(c.clone());
        }

        for (f, c) in signal_batch
            .schema()
            .fields()
            .iter()
            .zip(signal_batch.columns().iter())
        {
            fields.push(arrow::datatypes::Field::new(
                format!("signal_{}", f.name()),
                f.data_type().clone(),
                f.is_nullable(),
            ));
            columns.push(c.clone());
        }

        arrow::record_batch::RecordBatch::try_new(std::sync::Arc::new(arrow::datatypes::Schema::new(fields)), columns)
            .map_err(|_| KandError::InvalidData)
    }

    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(
        &mut self,
        _batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), KandError> {
        // Implementation for nested restoration...
        Err(KandError::InvalidData)
    }
}

/// Vectorized implementation of MACD for multiple independent streams.
#[cfg(feature = "arrow")]
#[derive(Clone)]
pub struct BatchMACD {
    fast_ema: ema::BatchEMA,
    slow_ema: ema::BatchEMA,
    signal_ema: ema::BatchEMA,
}

#[cfg(feature = "arrow")]
impl BatchMACD {
    /// Creates a new BatchMACD instance.
    pub fn new(
        opt_fast_period: usize,
        opt_slow_period: usize,
        opt_signal_period: usize,
        num_streams: usize,
    ) -> Result<Self, KandError> {
        let fast_ema = ema::BatchEMA::new_ext(opt_fast_period, num_streams, None)?;
        let slow_ema = ema::BatchEMA::new_ext(opt_slow_period, num_streams, None)?;
        let signal_ema = ema::BatchEMA::new_ext(opt_signal_period, num_streams, None)?;

        Ok(Self {
            fast_ema,
            slow_ema,
            signal_ema,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchMACD {
    type Input = TAArrowArray;
    type Output = (TAArrowArray, TAArrowArray, TAArrowArray);

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        use crate::ta::traits::BatchIndicator;
        use arrow::array::Array;

        let mut fast_clone = self.fast_ema.clone();
        let mut slow_clone = self.slow_ema.clone();
        let mut signal_clone = self.signal_ema.clone();

        let fast = fast_clone.next_batch((input.clone(),))?;
        let slow = slow_clone.next_batch((input,))?;

        let fast_values = fast.values();
        let slow_values = slow.values();

        let (ptr, macd_buffer) =
            crate::helper::buffer_pool::create_pooled_buffer(fast.len() * std::mem::size_of::<f64>());
        let macd_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut f64, fast.len()) };

        for i in 0..fast.len() {
            if fast_values[i].is_nan() || slow_values[i].is_nan() {
                macd_slice[i] = TAFloat::NAN;
            } else {
                macd_slice[i] = fast_values[i] - slow_values[i];
            }
        }

        let macd_arrow = TAArrowArray::new(macd_buffer.into(), None);
        let signal_arrow = signal_clone.next_batch((macd_arrow.clone(),))?;

        let signal_values = signal_arrow.values();
        let (ptr, hist_buffer) =
            crate::helper::buffer_pool::create_pooled_buffer(fast.len() * std::mem::size_of::<f64>());
        let hist_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut f64, fast.len()) };

        let macd_vals = macd_arrow.values();
        for i in 0..fast.len() {
            if signal_values[i].is_nan() {
                hist_slice[i] = TAFloat::NAN;
            } else {
                hist_slice[i] = macd_vals[i] - signal_values[i];
            }
        }

        // We also need to fix macd_arrow to be NaN if signal is NaN
        let (ptr, final_macd_buffer) =
            crate::helper::buffer_pool::create_pooled_buffer(fast.len() * std::mem::size_of::<f64>());
        let final_macd_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut f64, fast.len()) };

        for i in 0..fast.len() {
            if signal_values[i].is_nan() {
                final_macd_slice[i] = TAFloat::NAN;
            } else {
                final_macd_slice[i] = macd_vals[i];
            }
        }

        self.fast_ema = fast_clone;
        self.slow_ema = slow_clone;
        self.signal_ema = signal_clone;

        Ok((
            TAArrowArray::new(final_macd_buffer.into(), None),
            signal_arrow,
            TAArrowArray::new(hist_buffer.into(), None),
        ))
    }

    #[cfg(feature = "arrow")]
    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        Err(KandError::InvalidData)
    }

    #[cfg(feature = "arrow")]
    fn restore_from_record_batch(
        &mut self,
        _batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), KandError> {
        Err(KandError::InvalidData)
    }
}

/// MACD without input validation.
pub fn macd_raw(
    input: &[TAFloat],
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_signal_period: usize,
    output_macd: &mut [TAFloat],
    output_signal: &mut [TAFloat],
    output_hist: &mut [TAFloat],
) {
    let len = input.len();
    let mut fast_ema = vec![0.0; len];
    let mut slow_ema = vec![0.0; len];

    super::ema::ema_raw(input, opt_fast_period, None, &mut fast_ema);
    super::ema::ema_raw(input, opt_slow_period, None, &mut slow_ema);

    for i in 0..len {
        if fast_ema[i].is_nan() || slow_ema[i].is_nan() {
            output_macd[i] = TAFloat::NAN;
        } else {
            output_macd[i] = fast_ema[i] - slow_ema[i];
        }
    }

    // Signal is EMA of MACD
    // Find first valid MACD start
    let macd_start = opt_slow_period - 1;
    super::ema::ema_raw(
        &output_macd[macd_start..],
        opt_signal_period,
        None,
        &mut output_signal[macd_start..],
    );
    output_signal[..macd_start].fill(TAFloat::NAN);

    for i in 0..len {
        if output_macd[i].is_nan() || output_signal[i].is_nan() {
            output_hist[i] = TAFloat::NAN;
        } else {
            output_hist[i] = output_macd[i] - output_signal[i];
        }
    }

    // Functional macd also fills macd with NaN if signal is NaN?
    // Let's check TA-Lib. Usually MACD is valid when both EMAs are valid.
    // But the lookback says it's only valid when signal is valid.
    let lookback = opt_slow_period + opt_signal_period - 2;
    for i in 0..lookback {
        output_macd[i] = TAFloat::NAN;
    }
}

/// Calculates Moving Average Convergence Divergence (MACD).
pub fn macd(
    input: &[TAFloat],
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_signal_period: usize,
    output_macd: &mut [TAFloat],
    output_signal: &mut [TAFloat],
    output_hist: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input.len();
    let lookback = lookback(opt_fast_period, opt_slow_period, opt_signal_period)?;

    #[cfg(feature = "check")]
    {
        if len == 0 {
            return Err(KandError::InvalidData);
        }
        if len != output_macd.len() || len != output_signal.len() || len != output_hist.len() {
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

    macd_raw(
        input,
        opt_fast_period,
        opt_slow_period,
        opt_signal_period,
        output_macd,
        output_signal,
        output_hist,
    );

    Ok(())
}

/// Calculates MACD incrementally for a single value.
pub fn macd_inc(
    input_price: TAFloat,
    prev_fast_ema: TAFloat,
    prev_slow_ema: TAFloat,
    prev_signal_ema: TAFloat,
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_signal_period: usize,
) -> Result<(TAFloat, TAFloat, TAFloat), KandError> {
    let fast = super::ema::ema_inc(input_price, prev_fast_ema, opt_fast_period, None)?;
    let slow = super::ema::ema_inc(input_price, prev_slow_ema, opt_slow_period, None)?;
    let macd_val = fast - slow;
    let signal = super::ema::ema_inc(macd_val, prev_signal_ema, opt_signal_period, None)?;
    Ok((macd_val, signal, macd_val - signal))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper_multi!(
    macd_arrow,
    crate::ta::ohlcv::macd::macd_raw,
    inputs: { input },
    params: { opt_fast_period: usize, opt_slow_period: usize, opt_signal_period: usize },
    lookback_params: { opt_fast_period, opt_slow_period, opt_signal_period },
    outputs: { output_macd: crate::TAFloat, output_signal: crate::TAFloat, output_hist: crate::TAFloat },
    return_type: { crate::ta::types::TAArrowArray, crate::ta::types::TAArrowArray, crate::ta::types::TAArrowArray }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    #[test]
    fn test_stateful_macd() {
        let mut macd_state = StatefulMACD::new(2, 3, 2).unwrap();
        // fast: 2, slow: 3, signal: 2. lookback: 3+2-2=3. Need 4 values for first signal.
        assert!(macd_state.next(10.0).unwrap().0.is_nan());
        assert!(macd_state.next(11.0).unwrap().0.is_nan());
        assert!(macd_state.next(12.0).unwrap().0.is_nan());
        let (m, s, h) = macd_state.next(13.0).unwrap();
        assert!(!m.is_nan());
        assert!(!s.is_nan());
        assert!(!h.is_nan());
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_macd() {
        let mut batch_macd = BatchMACD::new(2, 3, 2, 2).unwrap();
        let input = TAArrowArray::from(vec![10.0, 20.0]);
        batch_macd.next_batch(input.clone()).unwrap();

        let input = TAArrowArray::from(vec![11.0, 21.0]);
        batch_macd.next_batch(input.clone()).unwrap();

        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let (m, _, _) = batch_macd.next_batch(input.clone()).unwrap();
        assert!(m.value(0).is_nan());

        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let (m, s, _) = batch_macd.next_batch(input.clone()).unwrap();
        assert!(!m.value(0).is_nan());
        assert!(!s.value(0).is_nan());
    }
}
