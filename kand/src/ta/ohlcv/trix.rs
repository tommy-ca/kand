use super::{ema, roc};
use crate::{KandError, TAFloat};

/// Calculates the lookback period required for TRIX calculation.
///
/// # Description
/// The lookback period represents the minimum number of data points needed before
/// the first valid TRIX value can be calculated. For TRIX, it is determined by
/// the triple EMA smoothing plus the Rate of Change lookback.
///
/// # Arguments
/// * `opt_period` - The smoothing period for EMAs. Must be >= 2.
///
/// # Returns
/// * `Result<usize, KandError>` - The required lookback period if successful
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
///
/// # Example
/// ```
/// use kand::ohlcv::trix;
/// let period = 14;
/// let lookback = trix::lookback(period).unwrap();
/// assert_eq!(lookback, 40); // 3 * (14 - 1) + 1
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(3 * (opt_period - 1) + 1)
}

/// Calculates TRIX without input validation.
pub fn trix_raw(
    input: &[TAFloat],
    opt_period: usize,
    output: &mut [TAFloat],
    ema1_output: &mut [TAFloat],
    ema2_output: &mut [TAFloat],
    ema3_output: &mut [TAFloat],
) {
    let len = input.len();
    let alpha = 2.0 / (opt_period + 1) as TAFloat;
    let lookback1 = opt_period - 1;
    let lookback2 = 2 * (opt_period - 1);
    let lookback3 = 3 * (opt_period - 1);

    // 1. First EMA
    let mut sum1 = 0.0;
    for i in 0..opt_period {
        sum1 += input[i];
    }
    let mut prev_ema1 = sum1 / opt_period as TAFloat;
    ema1_output[lookback1] = prev_ema1;

    for i in opt_period..len {
        prev_ema1 = (input[i] - prev_ema1).mul_add(alpha, prev_ema1);
        ema1_output[i] = prev_ema1;
    }

    // 2. Second EMA (EMA of EMA1)
    let mut sum2 = 0.0;
    for i in 0..opt_period {
        sum2 += ema1_output[lookback1 + i];
    }
    let mut prev_ema2 = sum2 / opt_period as TAFloat;
    ema2_output[lookback2] = prev_ema2;

    for i in lookback2 + 1..len {
        prev_ema2 = (ema1_output[i] - prev_ema2).mul_add(alpha, prev_ema2);
        ema2_output[i] = prev_ema2;
    }

    // 3. Third EMA (EMA of EMA2)
    let mut sum3 = 0.0;
    for i in 0..opt_period {
        sum3 += ema2_output[lookback2 + i];
    }
    let mut prev_ema3 = sum3 / opt_period as TAFloat;
    ema3_output[lookback3] = prev_ema3;

    for i in lookback3 + 1..len {
        prev_ema3 = (ema2_output[i] - prev_ema3).mul_add(alpha, prev_ema3);
        ema3_output[i] = prev_ema3;
    }

    // 4. ROC(1) of the triple-smoothed EMA
    for i in lookback3 + 1..len {
        let prev = ema3_output[i - 1];
        if prev != 0.0 {
            output[i] = (ema3_output[i] - prev) / prev * 100.0;
        } else {
            output[i] = TAFloat::NAN;
        }
    }
}

/// Calculates the Triple Exponential Average (TRIX) for a given price series.
///
/// # Description
/// TRIX is a momentum oscillator that shows the percentage rate of change of a triple exponentially smoothed moving average.
/// It is designed to filter out market noise and identify trends.
///
/// # Mathematical Formula
/// ```text
/// EMA1 = EMA(price, period)
/// EMA2 = EMA(EMA1, period)
/// EMA3 = EMA(EMA2, period)
/// TRIX = 100 * (EMA3_current - EMA3_previous) / EMA3_previous
/// ```
///
/// # Calculation Steps
/// 1. Calculate a n-period EMA of prices.
/// 2. Calculate a n-period EMA of the result from step 1.
/// 3. Calculate a n-period EMA of the result from step 2.
/// 4. Calculate a 1-period ROC (Rate of Change) of the result from step 3.
///
/// # Arguments
/// * `input` - Slice of input price values
/// * `opt_period` - Smoothing period for the EMAs (must be >= 2)
/// * `output` - Mutable slice to store TRIX results
/// * `ema1_output` - Mutable slice to store intermediate first EMA
/// * `ema2_output` - Mutable slice to store intermediate second EMA
/// * `ema3_output` - Mutable slice to store intermediate third EMA
///
/// # Returns
/// * `Result<(), KandError>` - Empty Ok value on success
///
/// # Errors
/// * `KandError::InvalidData` - Input slice is empty
/// * `KandError::LengthMismatch` - Output arrays don't match input length
/// * `KandError::InvalidParameter` - Period is less than 2
/// * `KandError::InsufficientData` - Input length is less than required lookback period
/// * `KandError::NaNDetected` - Input contains NaN values (when `check-nan` enabled)
///
/// # Example
/// ```
/// use kand::ohlcv::trix;
///
/// let input = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0];
/// let period = 2;
/// let mut output = vec![0.0; 6];
/// let mut ema1 = vec![0.0; 6];
/// let mut ema2 = vec![0.0; 6];
/// let mut ema3 = vec![0.0; 6];
///
/// trix::trix(
///     &input,
///     period,
///     &mut output,
///     &mut ema1,
///     &mut ema2,
///     &mut ema3,
/// )
/// .unwrap();
/// ```
pub fn trix(
    input: &[TAFloat],
    opt_period: usize,
    output: &mut [TAFloat],
    ema1_output: &mut [TAFloat],
    ema2_output: &mut [TAFloat],
    ema3_output: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        // Check if input array is empty
        if len == 0 {
            return Err(KandError::InvalidData);
        }

        // Check if input length is less than or equal to lookback period
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }

        // Check if output arrays have the same length as input
        if len != output.len()
            || len != ema1_output.len()
            || len != ema2_output.len()
            || len != ema3_output.len()
        {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        // Check if input contains NaN values
        for value in input {
            if value.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    trix_raw(input, opt_period, output, ema1_output, ema2_output, ema3_output);

    // Fill initial periods with NAN for all outputs
    for i in 0..lookback {
        output[i] = TAFloat::NAN;
    }
    for i in 0..opt_period - 1 {
        ema1_output[i] = TAFloat::NAN;
    }
    for i in 0..2 * (opt_period - 1) {
        ema2_output[i] = TAFloat::NAN;
    }
    for i in 0..3 * (opt_period - 1) {
        ema3_output[i] = TAFloat::NAN;
    }

    Ok(())
}

#[derive(Clone)]
pub struct StatefulTRIX {
    ema1: ema::StatefulEMA,
    ema2: ema::StatefulEMA,
    ema3: ema::StatefulEMA,
    roc: roc::StatefulROC,
}

impl StatefulTRIX {
    pub fn new(period: usize) -> Result<Self, KandError> {
        Ok(Self {
            ema1: ema::StatefulEMA::new_ext(period, None)?,
            ema2: ema::StatefulEMA::new_ext(period, None)?,
            ema3: ema::StatefulEMA::new_ext(period, None)?,
            roc: roc::StatefulROC::new(1)?,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulTRIX {
    type Input = (TAFloat,);
    type Output = (TAFloat,);

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        let mut ema1_clone = self.ema1.clone();
        let mut ema2_clone = self.ema2.clone();
        let mut ema3_clone = self.ema3.clone();
        let mut roc_clone = self.roc.clone();

        let e1 = ema1_clone.next(input)?;
        let e2 = ema2_clone.next((e1,))?;
        let e3 = ema3_clone.next((e2,))?;
        let trix = roc_clone.next((e3,))?;

        self.ema1 = ema1_clone;
        self.ema2 = ema2_clone;
        self.ema3 = ema3_clone;
        self.roc = roc_clone;

        Ok((trix,))
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

#[cfg(feature = "arrow")]
#[derive(Clone)]
pub struct BatchTRIX {
    ema1: ema::BatchEMA,
    ema2: ema::BatchEMA,
    ema3: ema::BatchEMA,
    roc: roc::BatchROC,
}

#[cfg(feature = "arrow")]
impl BatchTRIX {
    pub fn new(period: usize, num_streams: usize) -> Result<Self, KandError> {
        Ok(Self {
            ema1: ema::BatchEMA::new_ext(period, num_streams, None)?,
            ema2: ema::BatchEMA::new_ext(period, num_streams, None)?,
            ema3: ema::BatchEMA::new_ext(period, num_streams, None)?,
            roc: roc::BatchROC::new(1, num_streams)?,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchTRIX {
    type Input = (crate::ta::types::TAArrowArray,);
    type Output = (crate::ta::types::TAArrowArray,);

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        let mut ema1_clone = self.ema1.clone();
        let mut ema2_clone = self.ema2.clone();
        let mut ema3_clone = self.ema3.clone();
        let mut roc_clone = self.roc.clone();

        let e1 = ema1_clone.next_batch(input)?;
        let e2 = ema2_clone.next_batch((e1,))?;
        let e3 = ema3_clone.next_batch((e2,))?;
        let trix = roc_clone.next_batch((e3,))?;


        self.ema1 = ema1_clone;
        self.ema2 = ema2_clone;
        self.ema3 = ema3_clone;
        self.roc = roc_clone;

        Ok((trix,))
    }

    fn to_record_batch(&self) -> Result<arrow::record_batch::RecordBatch, KandError> {
        Err(KandError::InvalidData)
    }

    fn restore_from_record_batch(
        &mut self,
        _batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), KandError> {
        Err(KandError::InvalidData)
    }
}


/// Computes the next TRIX value incrementally without input validation.
#[inline]
pub fn trix_inc_raw(
    input: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    prev_ema3: TAFloat,
    prev_smoothed_ema3: TAFloat,
    opt_period: usize,
) -> (TAFloat, TAFloat, TAFloat, TAFloat) {
    let alpha = 2.0 / (opt_period + 1) as TAFloat;
    let ema1 = input.mul_add(alpha, prev_ema1 * (1.0 - alpha));
    let ema2 = ema1.mul_add(alpha, prev_ema2 * (1.0 - alpha));
    let ema3 = ema2.mul_add(alpha, prev_ema3 * (1.0 - alpha));
    let trix = 100.0 * (ema3 - prev_smoothed_ema3) / prev_smoothed_ema3;
    (trix, ema1, ema2, ema3)
}

/// Calculates TRIX value incrementally using previous EMA values
///
/// # Description
/// This function enables real-time TRIX calculation by using the previous triple-smoothed EMA values and latest price.
/// It avoids recalculating the entire series, making it efficient for streaming data.
///
/// # Arguments
/// * `input` - Latest price value to process
/// * `prev_ema1` - Previous value of first EMA
/// * `prev_ema2` - Previous value of second EMA
/// * `prev_ema3` - Previous value of third EMA
/// * `prev_smoothed_ema3` - Previous value of the third EMA (needed for ROC calculation)
/// * `opt_period` - Smoothing period for the EMAs (must be >= 2)
///
/// # Returns
/// * `Result<(TAFloat, TAFloat, TAFloat, TAFloat), KandError>` - Tuple containing:
///   - Current TRIX value
///   - Updated first EMA
///   - Updated second EMA
///   - Updated third EMA
///
/// # Errors
/// * `KandError::InvalidParameter` - Period is less than 2
/// * `KandError::NaNDetected` - Any input value is NaN (when `check-nan` enabled)
/// * `KandError::InvalidData` - If division by zero occurs during ROC calculation
///
/// # Example
/// ```
/// use kand::ohlcv::trix;
///
/// let price = 100.0;
/// let prev_ema1 = 98.0;
/// let prev_ema2 = 97.0;
/// let prev_ema3 = 96.0;
/// let prev_smoothed_ema3 = 95.5;
/// let period = 14;
///
/// let (trix_val, ema1, ema2, ema3) =
///     trix::trix_inc(price, prev_ema1, prev_ema2, prev_ema3, prev_smoothed_ema3, period).unwrap();
/// ```
pub fn trix_inc(
    input: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    prev_ema3: TAFloat,
    prev_smoothed_ema3: TAFloat,
    opt_period: usize,
) -> Result<(TAFloat, TAFloat, TAFloat, TAFloat), KandError> {
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
        if input.is_nan()
            || prev_ema1.is_nan()
            || prev_ema2.is_nan()
            || prev_ema3.is_nan()
            || prev_smoothed_ema3.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    // Check for division by zero
    if prev_smoothed_ema3 == 0.0 {
        return Err(KandError::InvalidData);
    }

    Ok(trix_inc_raw(
        input,
        prev_ema1,
        prev_ema2,
        prev_ema3,
        prev_smoothed_ema3,
        opt_period,
    ))
}

// Arrow wrapper
#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper_multi!(
    trix_arrow,
    crate::ta::ohlcv::trix::trix_raw,
    inputs: { input },
    params: { opt_period: usize },
    lookback_params: { opt_period },
    outputs: {
        output: TAFloat,
        ema1_output: TAFloat,
        ema2_output: TAFloat,
        ema3_output: TAFloat
    },
    return_type: {
        crate::ta::types::TAArrowArray,
        crate::ta::types::TAArrowArray,
        crate::ta::types::TAArrowArray,
        crate::ta::types::TAArrowArray
    }
);

#[cfg(test)]
mod tests {
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_trix_calculation() {
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let period = 5;
        let mut output = vec![0.0; input.len()];
        let mut ema1 = vec![0.0; input.len()];
        let mut ema2 = vec![0.0; input.len()];
        let mut ema3 = vec![0.0; input.len()];

        trix(
            &input,
            period,
            &mut output,
            &mut ema1,
            &mut ema2,
            &mut ema3,
        )
        .unwrap();

        let lookback = lookback(period).unwrap();
        for i in 0..lookback {
            assert!(output[i].is_nan());
        }

        let expected_trix = [
            -0.005733952289875416,
            -0.01755190956161911,
            -0.03320778213922027,
            -0.04913324862866834,
            -0.05772223210241969,
            -0.053924839634851056,
            -0.05012112393073105,
            -0.054424473844565266,
            -0.05663071979637735,
            -0.05054852862805311,
            -0.03729538305647722,
            -0.019927833141158972,
        ];

        for (i, expected) in expected_trix.iter().enumerate() {
            assert_relative_eq!(output[i + lookback], *expected, epsilon = 0.00000001);
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_trix_arrow() {
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let input_arrow = TAArrowArray::from(input);
        let period = 5;

        let (trix_arrow, _, _, _) = trix_arrow(&input_arrow, period).unwrap();

        assert_eq!(trix_arrow.len(), 25);
        let lookback = lookback(period).unwrap();

        for i in 0..lookback {
            assert!(trix_arrow.value(i).is_nan());
        }

        assert_relative_eq!(trix_arrow.value(24), -0.019927833141158972, epsilon = 0.00000001);
    }
}
