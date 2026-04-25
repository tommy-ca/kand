use crate::{KandError, TAFloat};

/// Returns the lookback period for ROCR100 without input validation.
#[inline]
pub const fn lookback_raw(opt_period: usize) -> usize {
    opt_period
}

/// Calculates the lookback period required for ROCR100 (Rate of Change Ratio * 100) calculation.
///
/// The lookback period equals the input parameter period since ROCR100 needs historical data points
/// to compare current prices with past prices.
///
/// # Arguments
/// * `opt_period` - The number of periods to look back for price comparison (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The required lookback period on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If `opt_period` is less than 2
///
/// # Example
/// ```
/// use kand::ohlcv::rocr100;
///
/// let opt_period = 10;
/// let lookback = rocr100::lookback(opt_period).unwrap();
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

/// Computes ROCR100 without input validation for high performance.
pub fn rocr100_raw(input_price: &[TAFloat], opt_period: usize, output_rocr100: &mut [TAFloat]) {
    let len = input_price.len();
    let lookback = lookback_raw(opt_period);

    // Calculate ROCR100 values
    for i in lookback..len {
        output_rocr100[i] = (input_price[i] / input_price[i - opt_period]) * 100.0;
    }
}

/// Calculates Rate of Change Ratio * 100 (ROCR100) for a price series.
///
/// ROCR100 is a momentum indicator that measures the percentage change in price over a specified period.
/// It compares the current price to a past price and expresses the ratio as a percentage.
/// Values above 100 indicate price increases, while values below 100 indicate price decreases.
///
/// # Mathematical Formula
/// ```text
/// ROCR100 = (Current Price / Price n periods ago) * 100
/// ```
///
/// # Calculation Principles
/// 1. For each data point, divide current price by price from n periods ago
/// 2. Multiply the ratio by 100 to get percentage
/// 3. First n values are set to NaN due to insufficient historical data
///
/// # Arguments
/// * `input_price` - Array of price values for calculation
/// * `opt_period` - Number of periods to look back (must be >= 2)
/// * `output_rocr100` - Array to store calculated ROCR100 values
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
/// use kand::ohlcv::rocr100;
///
/// let input_price = vec![10.0, 10.5, 11.2, 10.8, 11.5];
/// let opt_period = 2;
/// let mut output_rocr100 = vec![0.0; 5];
///
/// rocr100::rocr100(&input_price, opt_period, &mut output_rocr100).unwrap();
/// // First opt_period values are NaN
/// // Remaining values show percentage ratio between current and historical prices
/// ```
pub fn rocr100(
    input_price: &[TAFloat],
    opt_period: usize,
    output_rocr100: &mut [TAFloat],
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
        if len != output_rocr100.len() {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_price {
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    rocr100_raw(input_price, opt_period, output_rocr100);

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for value in output_rocr100.iter_mut().take(lookback) {
            *value = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Computes the next ROCR100 value incrementally without input validation.
#[inline]
pub fn rocr100_inc_raw(input: TAFloat, prev: TAFloat) -> TAFloat {
    (input / prev) * 100.0
}

/// Calculates a single ROCR100 value incrementally.
///
/// This function provides an optimized way to calculate ROCR100 in real-time scenarios
/// where only the latest value needs to be computed. It requires storing the historical
/// price value from `opt_period` periods ago.
///
/// # Arguments
/// * `input` - Current price value
/// * `prev` - Price value from `opt_period` periods ago
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The calculated ROCR100 value
///
/// # Errors
/// * `KandError::NaNDetected` - If any input value is NaN (when "`check-nan`" feature is enabled)
///
/// # Example
/// ```
/// use kand::ohlcv::rocr100;
///
/// let current_price = 15.0;
/// let price_10_periods_ago = 12.0;
///
/// // Calculate ROCR100 for current period
/// let rocr100_value = rocr100::rocr100_inc(current_price, price_10_periods_ago).unwrap();
/// // Result shows current price is 125% of price 10 periods ago
/// ```
pub fn rocr100_inc(input: TAFloat, prev: TAFloat) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check-nan")]
    {
        if input.is_nan() || prev.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(rocr100_inc_raw(input, prev))
}

// Arrow wrapper
#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    rocr100_arrow,
    crate::ta::ohlcv::rocr100::rocr100_raw,
    inputs: { input_price },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    ROCR100,
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
            Ok((price / old_val) * 100.0)
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
    fn test_stateful_rocr100() {
        let mut rocr100_state = StatefulROCR100::new(2).unwrap();
        assert!(rocr100_state.next((10.0,)).unwrap().is_nan());
        assert!(rocr100_state.next((10.5,)).unwrap().is_nan());
        assert_relative_eq!(rocr100_state.next((11.2,)).unwrap(), 112.0, epsilon = 0.0001);
        assert_relative_eq!(rocr100_state.next((10.8,)).unwrap(), 102.85714285714288, epsilon = 0.0001);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_rocr100() {
        let mut batch_rocr100 = BatchROCR100::new(2, 2).unwrap();
        let input = TAArrowArray::from(vec![10.0, 20.0]);

        // t0
        let out = batch_rocr100.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![10.5, 21.0]);
        let out = batch_rocr100.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![11.2, 22.4]);
        let out = batch_rocr100.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 112.0, epsilon = 0.0001);
        assert_relative_eq!(out.value(1), 112.0, epsilon = 0.0001);
    }

    #[test]
    fn test_rocr100_calculation() {
        let input_price = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6,
        ];
        let opt_period = 10;
        let mut output_rocr100 = vec![0.0; input_price.len()];

        rocr100(&input_price, opt_period, &mut output_rocr100).unwrap();

        // First 10 values should be NaN
        #[cfg(feature = "allow-nan")]
        for value in output_rocr100.iter().take(10) {
            assert!(value.is_nan());
        }

        // Compare with known values
        let expected_values = [
            99.883_575_978_032_78,
            100.024_133_055_471_95,
            100.061_948_185_173_93,
            99.973_557_008_814_31,
            99.740_772_849_366_86,
            99.394_688_920_027_45,
            99.421_920_983_558_12,
            99.323_440_722_344_05,
            99.634_634_179_603_15,
            99.544_972_672_781_07,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_rocr100[i + 10], *expected, epsilon = 0.0001);
        }

        // Test incremental calculation matches regular calculation
        for i in 11..input_price.len() {
            let result = rocr100_inc(input_price[i], input_price[i - opt_period]).unwrap();
            assert_relative_eq!(result, output_rocr100[i], epsilon = 0.0001);
        }
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_rocr100_arrow() {
        use crate::ta::types::TAArrowArray;
        let input_price = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6,
        ];
        let input_price_arrow = TAArrowArray::from(input_price);
        let opt_period = 10;

        let output_rocr100_arrow = rocr100_arrow(&input_price_arrow, opt_period).unwrap();

        // Compare with known values
        let expected_values = [
            99.883_575_978_032_78,
            100.024_133_055_471_95,
            100.061_948_185_173_93,
            99.973_557_008_814_31,
            99.740_772_849_366_86,
            99.394_688_920_027_45,
            99.421_920_983_558_12,
            99.323_440_722_344_05,
            99.634_634_179_603_15,
            99.544_972_672_781_07,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(
                output_rocr100_arrow.value(i + 10),
                *expected,
                epsilon = 0.0001
            );
        }
    }
}
