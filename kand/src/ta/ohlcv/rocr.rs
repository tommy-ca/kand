use crate::{KandError, TAFloat};

/// Returns the lookback period for Rate of Change Ratio (ROCR) without input validation.
#[inline]
pub const fn lookback_raw(opt_period: usize) -> usize {
    opt_period
}

/// Calculates the lookback period required for Rate of Change Ratio (ROCR) calculation.
///
/// Returns the number of historical data points needed for ROCR calculation, which equals the input period.
///
/// # Arguments
/// * `opt_period` - The time period used in ROCR calculation, must be >= 2
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
///
/// # Example
/// ```
/// use kand::ohlcv::rocr;
/// let lookback = rocr::lookback(10).unwrap();
/// assert_eq!(lookback, 10);
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(lookback_raw(opt_period))
}

/// Computes ROCR without input validation for high performance.
pub fn rocr_raw(input_price: &[TAFloat], opt_period: usize, output_rocr: &mut [TAFloat]) {
    let len = input_price.len();
    let lookback = lookback_raw(opt_period);

    // Calculate ROCR values
    for i in lookback..len {
        output_rocr[i] = input_price[i] / input_price[i - opt_period];
    }
}

/// Calculates Rate of Change Ratio (ROCR) for a price series.
///
/// ROCR is a momentum indicator that measures the percentage change in price over a specified
/// time period by comparing the current price to a previous price.
///
/// # Mathematical Formula
/// ```text
/// ROCR = Current Price / Price n periods ago
/// ```
///
/// # Calculation Principle
/// 1. For each data point after the lookback period:
///    - Divide current price by the price n periods ago
/// 2. Values before the lookback period are set to NaN
///
/// # Arguments
/// * `input_price` - Array of price values for calculation
/// * `opt_period` - Number of periods to look back (n), must be >= 2
/// * `output_rocr` - Array to store calculated ROCR values
///
/// # Returns
/// * `Result<(), KandError>` - Empty Ok value on success
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
/// use kand::ohlcv::rocr;
///
/// let input_price = vec![10.0, 10.5, 11.2, 10.8, 11.5];
/// let opt_period = 2;
/// let mut output_rocr = vec![0.0; 5];
///
/// rocr::rocr(&input_price, opt_period, &mut output_rocr).unwrap();
/// // First opt_period values will be NaN
/// // Remaining values show ratio between current and historical prices
/// ```
pub fn rocr(
    input_price: &[TAFloat],
    opt_period: usize,
    output_rocr: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_price.len();
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
        if len != output_rocr.len() {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_price {
            if price.is_null() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    rocr_raw(input_price, opt_period, output_rocr);

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for value in output_rocr.iter_mut().take(lookback) {
            *value = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Computes the next ROCR value incrementally without input validation.
#[inline]
pub fn rocr_inc_raw(input: TAFloat, prev: TAFloat) -> TAFloat {
    input / prev
}

/// Calculates a single ROCR value incrementally.
///
/// This function provides an optimized way to calculate ROCR for real-time data streams
/// by only computing the most recent value using the current price and historical price.
///
/// # Arguments
/// * `input` - Current price value
/// * `prev` - Price value from `opt_period` periods ago
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The calculated ROCR value
///
/// # Errors
/// * `KandError::NaNDetected` - If any input value is NaN (when "`check-nan`" feature is enabled)
///
/// # Example
/// ```
/// use kand::ohlcv::rocr;
///
/// let current_price = 15.0;
/// let historical_price = 12.0;
///
/// let rocr_value = rocr::rocr_inc(current_price, historical_price).unwrap();
/// // rocr_value will be 1.25 (15.0 / 12.0)
/// ```
pub fn rocr_inc(input: TAFloat, prev: TAFloat) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check-nan")]
    {
        if input.is_null() || prev.is_null() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(rocr_inc_raw(input, prev))
}

// Arrow wrapper
#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    rocr_arrow,
    crate::ta::ohlcv::rocr::rocr_raw,
    inputs: { input_price },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    ROCR,
    type: sliding_window,
    inputs: { price: TAFloat },
    params: { period: usize },
    state: { },
    init: |period| {
        ()
    },
    next: |state, (price)| {
        let old_val = state.__kand_window[state.__kand_cursor].0;
        state.__kand_window[state.__kand_cursor] = (price,);
        state.__kand_cursor = (state.__kand_cursor + 1) % state.period;

        if state.__kand_count <= state.period {
            Ok(TAFloat::NAN)
        } else if old_val == 0.0 {
            Ok(TAFloat::NAN)
        } else {
            Ok(price / old_val)
        }
    }
);

#[cfg(test)]
mod tests {
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    use super::*;

    #[test]
    fn test_stateful_rocr() {
        let mut rocr_state = StatefulROCR::new(2).unwrap();
        assert!(rocr_state.next((10.0,)).unwrap().is_nan());
        assert!(rocr_state.next((10.5,)).unwrap().is_nan());
        assert_relative_eq!(rocr_state.next((11.2,)).unwrap(), 1.12, epsilon = 0.0001);
        assert_relative_eq!(rocr_state.next((10.8,)).unwrap(), 1.02857, epsilon = 0.0001);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_rocr() {
        let mut batch_rocr = BatchROCR::new(2, 2).unwrap();
        let input = TAArrowArray::from(vec![10.0, 20.0]);

        // t0
        let out = batch_rocr.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![10.5, 21.0]);
        let out = batch_rocr.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![11.2, 22.4]);
        let out = batch_rocr.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 1.12, epsilon = 0.0001);
        assert_relative_eq!(out.value(1), 1.12, epsilon = 0.0001);
    }

    #[test]
    fn test_rocr_calculation() {
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6,
        ];
        let opt_period = 10;
        let mut output_rocr = vec![0.0; input.len()];

        rocr(&input, opt_period, &mut output_rocr).unwrap();

        // First 10 values should be NaN
        #[cfg(feature = "allow-nan")]
        for value in output_rocr.iter().take(10) {
            assert!(value.is_nan());
        }

        // Compare with known values
        let expected_values = [
            0.998_835_759_780_327_8,
            1.000_241_330_554_719_5,
            1.000_619_481_851_739_3,
            0.999_735_570_088_143_2,
            0.997_407_728_493_668_7,
            0.993_946_889_200_274_5,
            0.994_219_209_835_581_2,
            0.993_234_407_223_440_5,
            0.996_346_341_796_031_5,
            0.995_449_726_727_810_7,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_rocr[i + 10], *expected, epsilon = 0.0001);
        }

        // Test incremental calculation matches regular calculation
        for i in opt_period..input.len() {
            let result = rocr_inc(input[i], input[i - opt_period]).unwrap();
            assert_relative_eq!(result, output_rocr[i], epsilon = 0.0001);
        }
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_rocr_arrow() {
        use crate::ta::types::TAArrowArray;
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6,
        ];
        let input_arrow = TAArrowArray::from(input);
        let opt_period = 10;

        let output_rocr_arrow = rocr_arrow(&input_arrow, opt_period).unwrap();

        // Compare with known values
        let expected_values = [
            0.998_835_759_780_327_8,
            1.000_241_330_554_719_5,
            1.000_619_481_851_739_3,
            0.999_735_570_088_143_2,
            0.997_407_728_493_668_7,
            0.993_946_889_200_274_5,
            0.994_219_209_835_581_2,
            0.993_234_407_223_440_5,
            0.996_346_341_796_031_5,
            0.995_449_726_727_810_7,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_rocr_arrow.value(i + 10), *expected, epsilon = 0.0001);
        }
    }
}
