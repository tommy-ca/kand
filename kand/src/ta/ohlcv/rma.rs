use crate::{KandError, TAFloat, TAPeriod};

/// Returns the lookback period for RMA calculation without input validation.
#[inline]
pub const fn lookback_raw(opt_period: usize) -> TAPeriod {
    (opt_period - 1) as TAPeriod
}

/// Calculates the lookback period required for RMA calculation.
///
/// Returns the number of data points needed before RMA can start producing valid values.
/// The lookback period equals the period minus 1, since RMA requires a full period of data
/// to calculate the first value.
///
/// # Arguments
/// * `opt_period` - The period length used for RMA calculation (must be >= 2)
///
/// # Returns
/// * `Result<TAPeriod, KandError>` - The lookback period on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
///
/// # Examples
/// ```
/// use kand::ohlcv::rma;
/// let period = 14;
/// let lookback = rma::lookback(period).unwrap();
/// assert_eq!(lookback, 13); // lookback is period - 1
/// ```
pub const fn lookback(opt_period: usize) -> Result<TAPeriod, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(lookback_raw(opt_period))
}

/// Core calculation for Running Moving Average (RMA) without error checking.
///
/// # Arguments
/// * `input` - Array of price values to calculate RMA
/// * `opt_period` - The smoothing period
/// * `output_rma` - Array to store calculated RMA values
pub fn rma_raw(input: &[TAFloat], opt_period: usize, output_rma: &mut [TAFloat]) {
    let len = input.len();
    if len < opt_period {
        return;
    }

    // Calculate first SMA value
    let mut sum = 0.0;
    for &value in input.iter().take(opt_period) {
        sum += value;
    }
    let alpha = 1.0 / opt_period as TAFloat;
    output_rma[opt_period - 1] = sum / opt_period as TAFloat;

    // Calculate RMA for remaining values
    for i in opt_period..len {
        output_rma[i] = input[i].mul_add(alpha, output_rma[i - 1] * (1.0 - alpha));
    }

    // Fill initial values with NAN
    for value in output_rma.iter_mut().take(opt_period - 1) {
        *value = TAFloat::NAN;
    }
}

/// Calculates the Running Moving Average (RMA) for a price series.
///
/// RMA is a type of moving average that gives more weight to recent prices while still maintaining
/// some influence from all past prices. It is similar to EMA but uses a different smoothing factor.
///
/// # Mathematical Formula
/// ```text
/// RMA = (Current Price * α) + Previous RMA * (1 - α)
/// where α = 1/period
/// ```
///
/// # Calculation Steps
/// 1. Calculate initial SMA value using first `period` prices
/// 2. For remaining values, apply RMA formula using smoothing factor α = 1/period
/// 3. Fill initial values before period with NaN
///
/// # Arguments
/// * `input` - Array of price values to calculate RMA
/// * `opt_period` - The smoothing period (must be >= 2)
/// * `output_rma` - Array to store calculated RMA values
///
/// # Returns
/// * `Result<(), KandError>` - Empty result on success
///
/// # Errors
/// * `KandError::InvalidData` - If input array is empty
/// * `KandError::LengthMismatch` - If input and output arrays have different lengths
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::InsufficientData` - If input length is less than period
/// * `KandError::NaNDetected` - If input contains NaN values (with "`check-nan`" feature)
///
/// # Examples
/// ```
/// use kand::ohlcv::rma;
/// let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let period = 3;
/// let mut rma_values = vec![0.0; 5];
/// rma::rma(&prices, period, &mut rma_values).unwrap();
/// ```
pub fn rma(
    input: &[TAFloat],
    opt_period: usize,
    output_rma: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input.len();
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
        if len != output_rma.len() {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input.iter().take(len) {
            // NaN check
            if price.is_null() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    rma_raw(input, opt_period, output_rma);

    Ok(())
}

/// Core incremental calculation for Running Moving Average (RMA) without error checking.
#[inline]
pub fn rma_inc_raw(input_current: TAFloat, prev_rma: TAFloat, opt_period: usize) -> TAFloat {
    let alpha = 1.0 / opt_period as TAFloat;
    input_current.mul_add(alpha, prev_rma * (1.0 - alpha))
}

/// Calculates a single new RMA value incrementally.
///
/// This function enables real-time RMA calculation by computing the next value
/// using only the current price and previous RMA, without requiring historical data.
///
/// # Mathematical Formula
/// ```text
/// RMA = (Current Price * α) + Previous RMA * (1 - α)
/// where α = 1/period
/// ```
///
/// # Arguments
/// * `input_current` - The current price value
/// * `prev_rma` - The previous RMA value
/// * `opt_period` - The smoothing period (must be >= 2)
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The new RMA value on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::NaNDetected` - If any input is NaN (with "`check-nan`" feature)
///
/// # Examples
/// ```
/// use kand::ohlcv::rma;
/// let current_price = 10.0;
/// let prev_rma = 9.5;
/// let period = 14;
/// let new_rma = rma::rma_inc(current_price, prev_rma, period).unwrap();
/// ```
pub fn rma_inc(
    input_current: TAFloat,
    prev_rma: TAFloat,
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
        if input_current.is_null() || prev_rma.is_null() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(rma_inc_raw(input_current, prev_rma, opt_period))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    rma_arrow,
    crate::ta::ohlcv::rma::rma_raw,
    inputs: { input },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    use super::*;

    #[test]
    fn test_rma_calculation() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let opt_period = 5;
        let mut output_rma = vec![0.0; input.len()];

        rma(&input, opt_period, &mut output_rma).unwrap();

        // First (period-1) values should be NaN
        for value in output_rma.iter().take(opt_period - 1) {
            assert!(value.is_nan());
        }

        // First valid value should be SMA of first 5 values
        assert_relative_eq!(output_rma[4], 3.0, epsilon = 1e-12); // (1+2+3+4+5)/5 = 3.0

        // Subsequent values follow RMA formula with alpha = 1/5 = 0.2
        // RMA[5] = 6.0*0.2 + 3.0*0.8 = 1.2 + 2.4 = 3.6
        assert_relative_eq!(output_rma[5], 3.6, epsilon = 1e-12);

        // RMA[6] = 7.0*0.2 + 3.6*0.8 = 1.4 + 2.88 = 4.28
        assert_relative_eq!(output_rma[6], 4.28, epsilon = 1e-12);

        // RMA[7] = 8.0*0.2 + 4.28*0.8 = 1.6 + 3.424 = 5.024
        assert_relative_eq!(output_rma[7], 5.024, epsilon = 1e-12);

        // RMA[8] = 9.0*0.2 + 5.024*0.8 = 1.8 + 4.0192 = 5.8192
        assert_relative_eq!(output_rma[8], 5.8192, epsilon = 1e-12);

        // RMA[9] = 10.0*0.2 + 5.8192*0.8 = 2.0 + 4.65536 = 6.65536
        assert_relative_eq!(output_rma[9], 6.65536, epsilon = 1e-12);
    }

    #[test]
    fn test_rma_incremental() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let opt_period = 4;
        let mut output_rma = vec![0.0; input.len()];

        rma(&input, opt_period, &mut output_rma).unwrap();

        // Test incremental calculation matches regular calculation
        // Start from the first valid RMA value (after the lookback period)
        let lookback = lookback(opt_period).unwrap() as usize;

        // Start with the first valid RMA value
        let mut prev_rma = output_rma[lookback];

        // Test each incremental step
        for i in lookback + 1..input.len() {
            // Calculate next RMA using incremental method
            let next_rma = rma_inc(input[i], prev_rma, opt_period).unwrap();

            // Verify incremental result matches the regular calculation
            assert_relative_eq!(next_rma, output_rma[i], epsilon = 1e-12);

            // Update prev_rma for next iteration
            prev_rma = next_rma;
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_rma_arrow() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let period = 3;

        let input_arrow = TAArrowArray::from(input);
        let result = rma_arrow(&input_arrow, period).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result.value(0).is_nan());
        assert!(result.value(1).is_nan());
        assert_relative_eq!(result.value(2), 2.0, epsilon = 0.0001);
    }
}
