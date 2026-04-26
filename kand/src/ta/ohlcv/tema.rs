use crate::{KandError, TAFloat, ta::ohlcv::ema};

/// Calculates the lookback period required for Triple Exponential Moving Average (TEMA)
///
/// # Description
/// The lookback period represents the minimum number of data points needed before the first valid TEMA value
/// can be calculated. For TEMA, this equals 3 * (period - 1) due to the triple EMA calculation process.
///
/// # Arguments
/// * `opt_period` - The smoothing period used for TEMA calculation. Must be >= 2.
///
/// # Returns
/// * `Result<usize, KandError>` - The required lookback period if successful
///
/// # Errors
/// * `KandError::InvalidParameter` - Returned if period < 2
///
/// # Example
/// ```
/// use kand::ohlcv::tema;
/// let period = 14;
/// let lookback = tema::lookback(period).unwrap();
/// assert_eq!(lookback, 39); // 3 * (14 - 1)
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(3 * (opt_period - 1))
}

/// Calculates Triple Exponential Moving Average (TEMA) without input validation for high performance.
pub fn tema_raw(
    input: &[TAFloat],
    opt_period: usize,
    output_tema: &mut [TAFloat],
    output_ema1: &mut [TAFloat],
    output_ema2: &mut [TAFloat],
    output_ema3: &mut [TAFloat],
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
    output_ema1[lookback1] = prev_ema1;

    for i in opt_period..len {
        prev_ema1 = (input[i] - prev_ema1).mul_add(alpha, prev_ema1);
        output_ema1[i] = prev_ema1;
    }

    // 2. Second EMA (EMA of EMA1)
    let mut sum2 = 0.0;
    for i in 0..opt_period {
        sum2 += output_ema1[lookback1 + i];
    }
    let mut prev_ema2 = sum2 / opt_period as TAFloat;
    output_ema2[lookback2] = prev_ema2;

    for i in lookback2 + 1..len {
        prev_ema2 = (output_ema1[i] - prev_ema2).mul_add(alpha, prev_ema2);
        output_ema2[i] = prev_ema2;
    }

    // 3. Third EMA (EMA of EMA2)
    let mut sum3 = 0.0;
    for i in 0..opt_period {
        sum3 += output_ema2[lookback2 + i];
    }
    let mut prev_ema3 = sum3 / opt_period as TAFloat;
    output_ema3[lookback3] = prev_ema3;

    for i in lookback3 + 1..len {
        prev_ema3 = (output_ema2[i] - prev_ema3).mul_add(alpha, prev_ema3);
        output_ema3[i] = prev_ema3;
    }

    // 4. TEMA Calculation
    for i in lookback3..len {
        output_tema[i] = 3.0f64.mul_add(output_ema1[i], -(3.0 * output_ema2[i])) + output_ema3[i];
    }
}

/// Calculates Triple Exponential Moving Average (TEMA) for a price series
///
/// # Description
/// TEMA is an enhanced moving average designed to reduce lag while maintaining smoothing properties.
/// It applies triple exponential smoothing to put more weight on recent data and less on older data.
///
/// # Mathematical Formula
/// ```text
/// EMA1 = EMA(price, period)
/// EMA2 = EMA(EMA1, period)
/// EMA3 = EMA(EMA2, period)
/// TEMA = (3 × EMA1) - (3 × EMA2) + EMA3
/// ```
///
/// # Calculation Steps
/// 1. Calculate first EMA of the input prices
/// 2. Calculate second EMA using the first EMA values
/// 3. Calculate third EMA using the second EMA values
/// 4. Apply the TEMA formula to combine all three EMAs
///
/// # Arguments
/// * `input` - Slice of input price values
/// * `opt_period` - Smoothing period for calculations (must be >= 2)
/// * `output_tema` - Mutable slice to store TEMA results (first lookback values will be NaN)
/// * `output_ema1` - Mutable slice to store first EMA series
/// * `output_ema2` - Mutable slice to store second EMA series
/// * `output_ema3` - Mutable slice to store third EMA series
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
/// use kand::ohlcv::tema;
///
/// let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
/// let period = 3;
/// let mut output_tema = vec![0.0; input.len()];
/// let mut ema1 = vec![0.0; input.len()];
/// let mut ema2 = vec![0.0; input.len()];
/// let mut ema3 = vec![0.0; input.len()];
///
/// tema::tema(
///     &input,
///     period,
///     &mut output_tema,
///     &mut ema1,
///     &mut ema2,
///     &mut ema3,
/// )
/// .unwrap();
/// ```
pub fn tema(
    input: &[TAFloat],
    opt_period: usize,
    output_tema: &mut [TAFloat],
    output_ema1: &mut [TAFloat],
    output_ema2: &mut [TAFloat],
    output_ema3: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        // Check if input is empty
        if len == 0 {
            return Err(KandError::InvalidData);
        }

        // Check if input length is less than required lookback period
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }

        // Check if output arrays have the same length as input
        if len != output_tema.len()
            || len != output_ema1.len()
            || len != output_ema2.len()
            || len != output_ema3.len()
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

    tema_raw(
        input,
        opt_period,
        output_tema,
        output_ema1,
        output_ema2,
        output_ema3,
    );

    // Fill initial periods with NAN for all outputs
    for i in 0..lookback {
        output_tema[i] = TAFloat::NAN;
    }
    for i in 0..opt_period - 1 {
        output_ema1[i] = TAFloat::NAN;
    }
    for i in 0..2 * (opt_period - 1) {
        output_ema2[i] = TAFloat::NAN;
    }
    for i in 0..3 * (opt_period - 1) {
        output_ema3[i] = TAFloat::NAN;
    }

    Ok(())
}

#[derive(Clone)]
pub struct StatefulTEMA {
    _period: usize,
    ema1: ema::StatefulEMA,
    ema2: ema::StatefulEMA,
    ema3: ema::StatefulEMA,
}

impl StatefulTEMA {
    pub fn new(period: usize) -> Result<Self, KandError> {
        Ok(Self {
            _period: period,
            ema1: ema::StatefulEMA::new_ext(period, None)?,
            ema2: ema::StatefulEMA::new_ext(period, None)?,
            ema3: ema::StatefulEMA::new_ext(period, None)?,
        })
    }
}

impl crate::ta::traits::Indicator for StatefulTEMA {
    type Input = (TAFloat,);
    type Output = (TAFloat,);

    fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        let mut ema1_clone = self.ema1.clone();
        let mut ema2_clone = self.ema2.clone();
        let mut ema3_clone = self.ema3.clone();

        let e1 = ema1_clone.next(input)?;
        let e2 = ema2_clone.next((e1,))?;
        let e3 = ema3_clone.next((e2,))?;
        let tema = 3.0f64.mul_add(e1, -(3.0 * e2)) + e3;

        self.ema1 = ema1_clone;
        self.ema2 = ema2_clone;
        self.ema3 = ema3_clone;

        Ok((tema,))
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
pub struct BatchTEMA {
    _period: usize,
    ema1: ema::BatchEMA,
    ema2: ema::BatchEMA,
    ema3: ema::BatchEMA,
}

#[cfg(feature = "arrow")]
impl BatchTEMA {
    pub fn new(period: usize, num_streams: usize) -> Result<Self, KandError> {
        Ok(Self {
            _period: period,
            ema1: ema::BatchEMA::new_ext(period, num_streams, None)?,
            ema2: ema::BatchEMA::new_ext(period, num_streams, None)?,
            ema3: ema::BatchEMA::new_ext(period, num_streams, None)?,
        })
    }
}

#[cfg(feature = "arrow")]
impl crate::ta::traits::BatchIndicator for BatchTEMA {
    type Input = (crate::ta::types::TAArrowArray,);
    type Output = (crate::ta::types::TAArrowArray,);

    fn next_batch(&mut self, input: Self::Input) -> Result<Self::Output, KandError> {
        let mut ema1_clone = self.ema1.clone();
        let mut ema2_clone = self.ema2.clone();
        let mut ema3_clone = self.ema3.clone();

        let e1 = ema1_clone.next_batch(input)?;
        let e2 = ema2_clone.next_batch((e1.clone(),))?;
        let e3 = ema3_clone.next_batch((e2.clone(),))?;


        let len = e1.len();
        let e1_vals = e1.values();
        let e2_vals = e2.values();
        let e3_vals = e3.values();

        let (ptr, buffer) = crate::helper::buffer_pool::create_pooled_buffer(
            len * std::mem::size_of::<TAFloat>(),
        );
        let output = unsafe { std::slice::from_raw_parts_mut(ptr as *mut TAFloat, len) };

        for i in 0..len {
            output[i] = 3.0f64.mul_add(e1_vals[i], -(3.0 * e2_vals[i])) + e3_vals[i];
        }

        self.ema1 = ema1_clone;
        self.ema2 = ema2_clone;
        self.ema3 = ema3_clone;

        Ok((crate::ta::types::TAArrowArray::new(buffer.into(), None),))
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


/// Calculates TEMA value incrementally without input validation.
pub fn tema_inc_raw(
    input: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    prev_ema3: TAFloat,
    opt_period: usize,
) -> (TAFloat, TAFloat, TAFloat, TAFloat) {
    let alpha = 2.0 / (opt_period + 1) as TAFloat;
    let ema1 = input.mul_add(alpha, prev_ema1 * (1.0 - alpha));
    let ema2 = ema1.mul_add(alpha, prev_ema2 * (1.0 - alpha));
    let ema3 = ema2.mul_add(alpha, prev_ema3 * (1.0 - alpha));
    let tema = 3.0f64.mul_add(ema1, -(3.0 * ema2)) + ema3;
    (tema, ema1, ema2, ema3)
}

/// Calculates TEMA value incrementally using previous EMA values
///
/// # Description
/// This function enables real-time TEMA calculation by using the previous EMA values and latest price.
/// It avoids recalculating the entire series, making it efficient for streaming data.
///
/// # Arguments
/// * `input` - Latest price value to process
/// * `prev_ema1` - Previous value of first EMA
/// * `prev_ema2` - Previous value of second EMA
/// * `prev_ema3` - Previous value of third EMA
/// * `opt_period` - Smoothing period for calculations (must be >= 2)
///
/// # Returns
/// * `Result<(TAFloat, TAFloat, TAFloat, TAFloat), KandError>` - Tuple containing:
///   - Current TEMA value
///   - Updated first EMA
///   - Updated second EMA
///   - Updated third EMA
///
/// # Errors
/// * `KandError::InvalidParameter` - Period is less than 2
/// * `KandError::NaNDetected` - Any input value is NaN (when `check-nan` enabled)
///
/// # Example
/// ```
/// use kand::ohlcv::tema::tema_inc;
///
/// let new_price = 10.0;
/// let prev_ema1 = 9.0;
/// let prev_ema2 = 8.0;
/// let prev_ema3 = 7.0;
/// let period = 3;
///
/// let (tema, ema1, ema2, ema3) =
///     tema_inc(new_price, prev_ema1, prev_ema2, prev_ema3, period).unwrap();
/// ```
pub fn tema_inc(
    input: TAFloat,
    prev_ema1: TAFloat,
    prev_ema2: TAFloat,
    prev_ema3: TAFloat,
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
        if input.is_nan() || prev_ema1.is_nan() || prev_ema2.is_nan() || prev_ema3.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(tema_inc_raw(
        input, prev_ema1, prev_ema2, prev_ema3, opt_period,
    ))
}

// Arrow wrapper
#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper_multi!(
    tema_arrow,
    crate::ta::ohlcv::tema::tema_raw,
    inputs: { input },
    params: { opt_period: usize },
    lookback_params: { opt_period },
    outputs: {
        output_tema: TAFloat,
        output_ema1: TAFloat,
        output_ema2: TAFloat,
        output_ema3: TAFloat
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
    fn test_tema_calculation() {
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let period = 5;
        let mut output_tema = vec![0.0; input.len()];
        let mut output_ema1 = vec![0.0; input.len()];
        let mut output_ema2 = vec![0.0; input.len()];
        let mut output_ema3 = vec![0.0; input.len()];

        tema(
            &input,
            period,
            &mut output_tema,
            &mut output_ema1,
            &mut output_ema2,
            &mut output_ema3,
        )
        .unwrap();

        let lookback = lookback(period).unwrap();
        for i in 0..lookback {
            assert!(output_tema[i].is_nan());
        }

        let expected_tema = [
            35214.017805659714,
            35178.02412166619,
            35112.93264380731,
            35051.99395669962,
            35000.021905853544,
            34992.698578717165,
            35034.129121985054,
            35024.59996297049,
            34962.29030513636,
            34946.69299208543,
            34976.73223301734,
            35021.71538181302,
            35068.51163577639,
        ];

        for (i, expected) in expected_tema.iter().enumerate() {
            assert_relative_eq!(output_tema[i + lookback], *expected, epsilon = 0.00000001);
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_tema_arrow() {
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let input_arrow = TAArrowArray::from(input);
        let period = 5;

        let (tema_arrow, _, _, _) = tema_arrow(&input_arrow, period).unwrap();

        assert_eq!(tema_arrow.len(), 25);
        let lookback = lookback(period).unwrap();

        for i in 0..lookback {
            assert!(tema_arrow.value(i).is_nan());
        }

        assert_relative_eq!(tema_arrow.value(24), 35068.51163577639, epsilon = 0.00000001);
    }
}
