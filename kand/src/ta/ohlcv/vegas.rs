use super::ema;
use crate::{KandError, TAFloat};

/// Returns the lookback period required for VEGAS (Volume and EMA Guided Adaptive Scaling) calculation
///
/// # Returns
/// * `Result<usize, KandError>` - The number of data points needed before first valid output (675)
///
/// # Errors
/// * `KandError::InvalidData` - If input data is empty
///
/// # Example
/// ```rust
/// use kand::ohlcv::vegas;
///
/// let lookback = vegas::lookback().unwrap();
/// assert_eq!(lookback, 675);
/// ```
pub const fn lookback() -> Result<usize, KandError> {
    Ok(676 - 1) // Longest EMA period - 1
}

/// Calculates VEGAS (Volume and EMA Guided Adaptive Scaling) without input validation.
pub fn vegas_raw(
    input_price: &[TAFloat],
    output_channel_upper: &mut [TAFloat],
    output_channel_lower: &mut [TAFloat],
    output_boundary_upper: &mut [TAFloat],
    output_boundary_lower: &mut [TAFloat],
) {
    // Calculate EMAs
    let _ = ema::ema(input_price, 144, None, output_channel_upper); // Channel upper - EMA(144)
    let _ = ema::ema(input_price, 169, None, output_channel_lower); // Channel lower - EMA(169)
    let _ = ema::ema(input_price, 576, None, output_boundary_upper); // Boundary upper - EMA(576)
    let _ = ema::ema(input_price, 676, None, output_boundary_lower); // Boundary lower - EMA(676)
}

/// Calculates VEGAS (Volume and EMA Guided Adaptive Scaling) indicator for the entire price array
///
/// # Description
/// VEGAS is a trend following indicator that uses multiple EMAs to define channels and boundaries.
/// It helps identify trend strength and potential trend changes through the spacing between EMAs.
///
/// # Mathematical Formula
/// The indicator consists of 4 EMAs with different periods:
/// ```text
/// Channel Upper = EMA(price, 144)
/// Channel Lower = EMA(price, 169)
/// Boundary Upper = EMA(price, 576)
/// Boundary Lower = EMA(price, 676)
///
/// Where EMA is calculated as:
/// EMA = Price * (2 / (n + 1)) + Previous_EMA * (1 - (2 / (n + 1)))
/// ```
///
/// # Parameters
/// * `input_price` - Array of price values
/// * `output_channel_upper` - Output array for upper channel (EMA 144)
/// * `output_channel_lower` - Output array for lower channel (EMA 169)
/// * `output_boundary_upper` - Output array for upper boundary (EMA 576)
/// * `output_boundary_lower` - Output array for lower boundary (EMA 676)
///
/// # Returns
/// * `Result<(), KandError>` - Ok if calculation succeeds
///
/// # Errors
/// * `KandError::InvalidData` - If input array is empty
/// * `KandError::LengthMismatch` - If output arrays have different lengths than input
/// * `KandError::InsufficientData` - If input length < 676
/// * `KandError::NaNDetected` - If any input value is NaN
///
/// # Example
/// ```rust
/// use kand::ohlcv::vegas;
///
/// let input_price = vec![10.0; 1000];
/// let mut channel_upper = vec![0.0; 1000];
/// let mut channel_lower = vec![0.0; 1000];
/// let mut boundary_upper = vec![0.0; 1000];
/// let mut boundary_lower = vec![0.0; 1000];
///
/// vegas::vegas(
///     &input_price,
///     &mut channel_upper,
///     &mut channel_lower,
///     &mut boundary_upper,
///     &mut boundary_lower,
/// )
/// .unwrap();
/// ```
pub fn vegas(
    input_price: &[TAFloat],
    output_channel_upper: &mut [TAFloat],
    output_channel_lower: &mut [TAFloat],
    output_boundary_upper: &mut [TAFloat],
    output_boundary_lower: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_price.len();
    let lookback = lookback()?;

    #[cfg(feature = "check")]
    {
        // Empty data check
        if len == 0 {
            return Err(KandError::InvalidData);
        }

        // Length consistency check
        if output_channel_upper.len() != len
            || output_channel_lower.len() != len
            || output_boundary_upper.len() != len
            || output_boundary_lower.len() != len
        {
            return Err(KandError::LengthMismatch);
        }

        // Data sufficiency check
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_price {
            // NaN check
            if price.is_null() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    vegas_raw(
        input_price,
        output_channel_upper,
        output_channel_lower,
        output_boundary_upper,
        output_boundary_lower,
    );

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for i in 0..lookback {
            output_channel_upper[i] = TAFloat::NAN;
            output_channel_lower[i] = TAFloat::NAN;
            output_boundary_upper[i] = TAFloat::NAN;
            output_boundary_lower[i] = TAFloat::NAN;
        }
    }

    Ok(())
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper_multi!(
    vegas_arrow,
    crate::ta::ohlcv::vegas::vegas_raw,
    inputs: { input_price },
    params: {},
    lookback_params: {},
    outputs: {
        output_channel_upper: TAFloat,
        output_channel_lower: TAFloat,
        output_boundary_upper: TAFloat,
        output_boundary_lower: TAFloat
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
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_vegas_calculation() {
        let input_price = vec![100.0; 700];
        let mut channel_upper = vec![0.0; 700];
        let mut channel_lower = vec![0.0; 700];
        let mut boundary_upper = vec![0.0; 700];
        let mut boundary_lower = vec![0.0; 700];

        vegas(
            &input_price,
            &mut channel_upper,
            &mut channel_lower,
            &mut boundary_upper,
            &mut boundary_lower,
        )
        .unwrap();

        let lookback = lookback().unwrap();

        #[cfg(feature = "allow-nan")]
        {
            for i in 0..lookback {
                assert!(channel_upper[i].is_nan());
                assert!(channel_lower[i].is_nan());
                assert!(boundary_upper[i].is_nan());
                assert!(boundary_lower[i].is_nan());
            }
        }

        // After lookback, values should be 100.0 for constant input
        assert_relative_eq!(channel_upper[lookback], 100.0);
        assert_relative_eq!(channel_lower[lookback], 100.0);
        assert_relative_eq!(boundary_upper[lookback], 100.0);
        assert_relative_eq!(boundary_lower[lookback], 100.0);
    }

    #[test]
    fn test_vegas_inc() {
        let current_price = 100.0;
        let prev_values = (100.0, 100.0, 100.0, 100.0);

        let new_values = vegas_inc(
            current_price,
            prev_values.0,
            prev_values.1,
            prev_values.2,
            prev_values.3,
        )
        .unwrap();

        assert_relative_eq!(new_values.0, 100.0);
        assert_relative_eq!(new_values.1, 100.0);
        assert_relative_eq!(new_values.2, 100.0);
        assert_relative_eq!(new_values.3, 100.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_vegas_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_price = vec![100.0; 700];
        let input_arrow = TAArrowArray::from(input_price);

        let (upper, lower, b_upper, b_lower) = vegas_arrow(&input_arrow).unwrap();

        assert_eq!(upper.len(), 700);
        let lookback = lookback().unwrap();

        #[cfg(feature = "allow-nan")]
        {
            for i in 0..lookback {
                assert!(upper.is_null(i));
                assert!(lower.is_null(i));
                assert!(b_upper.is_null(i));
                assert!(b_lower.is_null(i));
            }
        }

        assert_relative_eq!(upper.value(lookback), 100.0);
    }
}

/// Calculates latest VEGAS indicator values incrementally for real-time updates
///
/// # Description
/// Provides optimized calculation of latest VEGAS values for real-time price updates
/// without recalculating the entire series. This is particularly useful for streaming data.
///
/// # Mathematical Formula
/// ```text
/// For each EMA:
/// EMA_current = Price * multiplier + Previous_EMA * (1 - multiplier)
/// where multiplier = 2 / (period + 1)
/// ```
///
/// # Parameters
/// * `input_price` - Current price value
/// * `prev_channel_upper` - Previous EMA(144) value
/// * `prev_channel_lower` - Previous EMA(169) value
/// * `prev_boundary_upper` - Previous EMA(576) value
/// * `prev_boundary_lower` - Previous EMA(676) value
///
/// # Returns
/// * `Result<(TAFloat,TAFloat,TAFloat,TAFloat), KandError>` - Tuple of (`channel_upper`, `channel_lower`, `boundary_upper`, `boundary_lower`)
///
/// # Errors
/// * `KandError::NaNDetected` - If any input value is NaN
///
/// # Example
/// ```rust
/// use kand::ohlcv::vegas;
///
/// let current_price = 100.0;
/// let prev_values = (98.0, 97.5, 96.0, 95.5);
///
/// let new_values = vegas::vegas_inc(
///     current_price,
///     prev_values.0,
///     prev_values.1,
///     prev_values.2,
///     prev_values.3,
/// )
/// .unwrap();
/// ```
pub fn vegas_inc(
    input_price: TAFloat,
    prev_channel_upper: TAFloat,
    prev_channel_lower: TAFloat,
    prev_boundary_upper: TAFloat,
    prev_boundary_lower: TAFloat,
) -> Result<(TAFloat, TAFloat, TAFloat, TAFloat), KandError> {
    #[cfg(feature = "check-nan")]
    {
        // NaN check
        if input_price.is_null()
            || prev_channel_upper.is_null()
            || prev_channel_lower.is_null()
            || prev_boundary_upper.is_null()
            || prev_boundary_lower.is_null()
        {
            return Err(KandError::NaNDetected);
        }
    }

    let channel_upper = ema::ema_inc(input_price, prev_channel_upper, 144, None)?;
    let channel_lower = ema::ema_inc(input_price, prev_channel_lower, 169, None)?;
    let boundary_upper = ema::ema_inc(input_price, prev_boundary_upper, 576, None)?;
    let boundary_lower = ema::ema_inc(input_price, prev_boundary_lower, 676, None)?;

    Ok((channel_upper, channel_lower, boundary_upper, boundary_lower))
}
