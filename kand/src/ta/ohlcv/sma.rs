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

/// Calculates SMA without input validation.
pub fn sma_raw(input: &[TAFloat], opt_period: TAPeriod, output: &mut [TAFloat]) {
    let mut sum = 0.0;
    for i in 0..opt_period {
        sum += input[i];
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
    for i in 0..lookback {
        output[i] = TAFloat::NAN;
    }

    Ok(())
}

/// Calculates SMA incrementally for a single value without validation.
#[inline]
#[must_use]
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
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    #[cfg(feature = "arrow")]
    fn test_sma() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let input_arrow = TAArrowArray::from(input);
        let period = 3;

        let result = sma_arrow(&input_arrow, period).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result.is_nan(0));
        assert!(result.is_nan(1));
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
        assert!(result.is_nan(0));
        assert!(result.is_nan(1));
        assert_relative_eq!(result.value(2), 2.0);
        assert_relative_eq!(result.value(3), 3.0);
        assert_relative_eq!(result.value(4), 4.0);
    }
}
