use crate::{EPSILON, KandError, TAFloat};

/// Calculates the lookback period required for Maximum Value calculation.
///
/// The lookback period represents the number of data points needed before the first valid output
/// can be calculated. For Maximum Value, this equals the period minus one.
///
/// # Arguments
/// * `opt_period` - The time period for calculation (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period (period - 1) on success
///
/// # Errors
/// * Returns `KandError::InvalidParameter` if period is less than 2
///
/// # Example
/// ```
/// use kand::stats::max;
/// let period = 14;
/// let lookback = max::lookback(period).unwrap();
/// assert_eq!(lookback, 13); // lookback is period - 1
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period - 1)
}

/// Calculates Max without input validation for high performance.
pub fn max_raw(input_prices: &[TAFloat], opt_period: usize, output_max: &mut [TAFloat]) {
    let len = input_prices.len();
    let lookback = opt_period - 1;

    for i in lookback..len {
        let mut max_val = input_prices[i - lookback];
        for price in input_prices.iter().take(i + 1).skip(i - lookback + 1) {
            if *price > max_val {
                max_val = *price;
            }
        }
        output_max[i] = max_val;
    }
}

/// Calculates Maximum Value for a series of prices over a specified period.
///
/// # Calculation Principle
/// For each period, finds the highest price value within that period by comparing
/// all values in the window.
///
/// # Mathematical Formula
/// ```text
/// MAX[i] = max(price[i-n+1], price[i-n+2], ..., price[i])
/// ```
/// Where:
/// - n is the period
/// - i is the current index
///
/// # Arguments
/// * `input_prices` - Array of input price values
/// * `opt_period` - The time period for calculation (must be >= 2)
/// * `output_max` - Array to store calculated MAX values (first period-1 values are NaN)
///
/// # Returns
/// * `Result<(), KandError>` - Empty result on success
///
/// # Errors
/// * Returns `KandError::InvalidData` if input array is empty
/// * Returns `KandError::LengthMismatch` if output length doesn't match input
/// * Returns `KandError::InvalidParameter` if period is less than 2
/// * Returns `KandError::InsufficientData` if input length is less than period
/// * Returns `KandError::NaNDetected` if any input value is NaN (with "`check-nan`" feature)
///
/// # Example
/// ```
/// use kand::stats::max;
/// let prices = vec![1.0, 2.0, 3.0, 2.5, 4.0];
/// let period = 3;
/// let mut max_values = vec![0.0; 5];
///
/// max::max(&prices, period, &mut max_values).unwrap();
/// // max_values = [NaN, NaN, 3.0, 3.0, 4.0]
/// ```
pub fn max(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_max: &mut [TAFloat],
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
        if output_max.len() != len {
            return Err(KandError::LengthMismatch);
        }

        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        // NaN check
        for price in input_prices {
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    max_raw(input_prices, opt_period, output_max);

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for value in output_max.iter_mut().take(lookback) {
            *value = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Calculates the next Max value without input validation.
#[must_use]
pub fn max_inc_raw(
    input_price: TAFloat,
    prev_max: TAFloat,
    input_old_price: TAFloat,
    _opt_period: usize,
) -> TAFloat {
    if input_price >= prev_max || (prev_max - input_old_price).abs() < EPSILON {
        input_price
    } else {
        prev_max
    }
}

/// Calculates the latest Maximum Value incrementally using previous results.
///
/// This function provides an optimized way to calculate the latest MAX value
/// by using the previous MAX value and only considering the new and removed prices.
///
/// # Arguments
/// * `input_price` - The newest price value to include in calculation
/// * `prev_max` - The previous period's MAX value
/// * `input_old_price` - The oldest price value being removed from the period
/// * `opt_period` - The time period for calculation (must be >= 2)
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The new MAX value on success
///
/// # Errors
/// * Returns `KandError::InvalidParameter` if period is less than 2
/// * Returns `KandError::NaNDetected` if any input value is NaN (with "`check-nan`" feature)
///
/// # Example
/// ```
/// use kand::stats::max;
/// let new_price = 10.5;
/// let prev_max = 11.0;
/// let old_price = 9.0;
/// let period = 14;
///
/// let new_max = max::max_inc(new_price, prev_max, old_price, period).unwrap();
/// assert_eq!(new_max, 11.0);
/// ```
pub fn max_inc(
    input_price: TAFloat,
    prev_max: TAFloat,
    input_old_price: TAFloat,
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
        if input_price.is_nan() || prev_max.is_nan() || input_old_price.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(max_inc_raw(
        input_price,
        prev_max,
        input_old_price,
        opt_period,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    max_arrow,
    crate::ta::stats::max::max_raw,
    inputs: { input_prices },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use arrow::array::Array;
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_max_calculation() {
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let opt_period = 14;
        let mut output_max = vec![0.0; input_close.len()];

        max(&input_close, opt_period, &mut output_max).unwrap();

        // First 13 values should be NaN
        #[cfg(feature = "allow-nan")]
        for value in output_max.iter().take(13) {
            assert!(value.is_nan());
        }

        // Compare with known values
        let expected_values = [
            35254.6, 35254.6, 35254.6, 35254.6, 35254.6, 35254.6, 35251.9, 35251.9, 35229.9,
            35229.9, 35229.9, 35229.9,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_max[i + 13], *expected, epsilon = 0.0001);
        }

        // Now test incremental calculation matches regular calculation
        let mut prev_max = output_max[13]; // First valid max value

        // Test each incremental step
        for i in 14..19 {
            let result = max_inc(
                input_close[i],
                prev_max,
                input_close[i - opt_period],
                opt_period,
            )
            .unwrap();
            assert_relative_eq!(result, output_max[i], epsilon = 0.0001);
            prev_max = result;
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_max_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let input_arrow = TAArrowArray::from(input_close.clone());
        let opt_period = 14;

        let result = max_arrow(&input_arrow, opt_period).unwrap();

        assert_eq!(result.len(), input_close.len());

        let mut out = vec![0.0; input_close.len()];
        max(&input_close, opt_period, &mut out).unwrap();

        for i in 0..input_close.len() {
            if i < 13 {
                #[cfg(feature = "allow-nan")]
                assert!(result.value(i).is_nan());
            } else {
                assert_relative_eq!(result.value(i), out[i], epsilon = 0.0001);
            }
        }
    }
}
