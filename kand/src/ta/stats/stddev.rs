use crate::{KandError, TAFloat, ta::stats::var};

#[cfg(feature = "arrow")]
use crate::ta::types::TAArrowArray;

/// Calculates the lookback period required for Standard Deviation calculation.
///
/// The lookback period represents the number of data points needed before the first valid output
/// can be generated. For Standard Deviation, this equals the period minus 1.
///
/// # Arguments
/// * `opt_period` - The time period for Standard Deviation calculation (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period (period - 1) on success, or error on failure
///
/// # Errors
/// * Returns `KandError::InvalidParameter` if period is less than 2
///
/// # Example
/// ```
/// use kand::stats::stddev;
/// let period = 14;
/// let lookback = stddev::lookback(period).unwrap();
/// assert_eq!(lookback, 13); // lookback is period - 1
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    var::lookback(opt_period)
}

/// Calculates Standard Deviation without input validation for high performance.
pub fn stddev_raw(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_stddev: &mut [TAFloat],
    output_sum: &mut [TAFloat],
    output_sum_sq: &mut [TAFloat],
) {
    let len = input_prices.len();
    let lookback = opt_period - 1;

    // Calculate variance first
    var::var_raw(
        input_prices,
        opt_period,
        output_stddev,
        output_sum,
        output_sum_sq,
    );

    // Take square root to get standard deviation
    for value in output_stddev.iter_mut().take(len).skip(lookback) {
        *value = value.sqrt();
    }
}

/// Calculates Standard Deviation for a price series.
///
/// Standard Deviation measures the dispersion of values from their mean over a specified period.
/// It is calculated by taking the square root of the variance.
///
/// # Mathematical Formula
/// ```text
/// STDDEV = sqrt(VAR)
/// where:
/// VAR = sum((x - mean)^2) / n
/// mean = sum(x) / n
/// ```
/// Where:
/// - x: Each value in the dataset
/// - n: Time period
///
/// # Arguments
/// * `input_prices` - Array of input values to calculate Standard Deviation
/// * `opt_period` - The time period for calculation (must be >= 2)
/// * `output_stddev` - Array to store calculated Standard Deviation values
/// * `output_sum` - Array to store running sum values
/// * `output_sum_sq` - Array to store running sum of squares values
///
/// # Returns
/// * `Result<(), KandError>` - Ok(()) on success, or error on failure
///
/// # Errors
/// * Returns `KandError::InvalidData` if input array is empty
/// * Returns `KandError::LengthMismatch` if output arrays don't match input length
/// * Returns `KandError::InvalidParameter` if period is less than 2
/// * Returns `KandError::InsufficientData` if input length <= lookback period
/// * Returns `KandError::NaNDetected` if any input value is NaN (when `check-nan` enabled)
///
/// # Example
/// ```
/// use kand::stats::stddev;
/// let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let period = 3;
/// let mut output_stddev = vec![0.0; 5];
/// let mut output_sum = vec![0.0; 5];
/// let mut output_sum_sq = vec![0.0; 5];
///
/// stddev::stddev(
///     &input,
///     period,
///     &mut output_stddev,
///     &mut output_sum,
///     &mut output_sum_sq,
/// )
/// .unwrap();
/// ```
pub fn stddev(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_stddev: &mut [TAFloat],
    output_sum: &mut [TAFloat],
    output_sum_sq: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_prices.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        if len == 0 {
            return Err(KandError::InvalidData);
        }
        if output_stddev.len() != len || output_sum.len() != len || output_sum_sq.len() != len {
            return Err(KandError::LengthMismatch);
        }
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_prices {
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    stddev_raw(
        input_prices,
        opt_period,
        output_stddev,
        output_sum,
        output_sum_sq,
    );

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for i in 0..lookback {
            output_stddev[i] = TAFloat::NAN;
            output_sum[i] = TAFloat::NAN;
            output_sum_sq[i] = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Calculates the latest Standard Deviation value incrementally without input validation
#[must_use]
pub fn stddev_inc_raw(
    input_price: TAFloat,
    prev_sum: TAFloat,
    prev_sum_sq: TAFloat,
    input_old_price: TAFloat,
    opt_period: usize,
) -> (TAFloat, TAFloat, TAFloat) {
    let (var, new_sum, new_sum_sq) =
        var::var_inc_raw(input_price, prev_sum, prev_sum_sq, input_old_price, opt_period);

    (var.sqrt(), new_sum, new_sum_sq)
}

/// Calculates the latest Standard Deviation value incrementally.
///
/// # Arguments
/// * `input_price` - The latest price value to include in calculation
/// * `prev_sum` - Previous sum of values in the period
/// * `prev_sum_sq` - Previous sum of squared values in the period
/// * `input_old_price` - Price value to remove from the period
/// * `opt_period` - The time period for calculation (must be >= 2)
///
/// # Returns
/// * `Result<(TAFloat, TAFloat, TAFloat), KandError>` - Tuple containing:
///   - Latest Standard Deviation value
///   - New sum
///   - New sum of squares
///
/// # Errors
/// * Returns `KandError::InvalidParameter` if period is less than 2
/// * Returns `KandError::NaNDetected` if any input value is NaN (when `check-nan` enabled)
///
/// # Example
/// ```
/// use kand::stats::stddev;
/// let (stddev, new_sum, new_sum_sq) = stddev::stddev_inc(
///     10.0,   // new price
///     100.0,  // previous sum
///     1050.0, // previous sum of squares
///     8.0,    // old price to remove
///     14,     // period
/// )
/// .unwrap();
/// ```
pub fn stddev_inc(
    input_price: TAFloat,
    prev_sum: TAFloat,
    prev_sum_sq: TAFloat,
    input_old_price: TAFloat,
    opt_period: usize,
) -> Result<(TAFloat, TAFloat, TAFloat), KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        if input_price.is_nan()
            || prev_sum.is_nan()
            || prev_sum_sq.is_nan()
            || input_old_price.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(stddev_inc_raw(
        input_price,
        prev_sum,
        prev_sum_sq,
        input_old_price,
        opt_period,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper_multi!(
    stddev,
    crate::ta::stats::stddev::stddev_raw,
    inputs: { input_prices },
    params: { opt_period: usize },
    outputs: { output_stddev, output_sum, output_sum_sq }
);

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_stddev_calculation() {
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let opt_period = 14;
        let mut output_stddev = vec![0.0; input_close.len()];
        let mut output_sum = vec![0.0; input_close.len()];
        let mut output_sum_sq = vec![0.0; input_close.len()];

        stddev(
            &input_close,
            opt_period,
            &mut output_stddev,
            &mut output_sum,
            &mut output_sum_sq,
        )
        .unwrap();

        // First 13 values should be NaN
        #[cfg(feature = "allow-nan")]
        for i in 0..13 {
            assert!(output_stddev[i].is_nan());
            assert!(output_sum[i].is_nan());
            assert!(output_sum_sq[i].is_nan());
        }

        // Compare with known values
        let expected_values = [
            28.040_929_452_086_566,
            40.126_741_172_470_275,
            55.432_097_183_551_12,
            72.497_236_047_872_85,
            82.691_165_345_452_96,
            85.326_748_921_912_07,
            85.513_580_494_380_46,
            96.234_748_328_008_62,
            96.370_323_146_837_6,
            94.344_639_592_305_9,
            89.879_298_030_442_56,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_stddev[i + 13], *expected, epsilon = 0.0001);
        }

        // Test incremental calculation
        let mut prev_sum = output_sum[13];
        let mut prev_sum_sq = output_sum_sq[13];

        // Test each incremental step
        for i in 14..19 {
            let (stddev, new_sum, new_sum_sq) = stddev_inc(
                input_close[i],
                prev_sum,
                prev_sum_sq,
                input_close[i - opt_period],
                opt_period,
            )
            .unwrap();
            assert_relative_eq!(stddev, output_stddev[i], epsilon = 0.0001);
            prev_sum = new_sum;
            prev_sum_sq = new_sum_sq;
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_stddev_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let input_arrow = TAArrowArray::from(input_close.clone());
        let opt_period = 14;

        let (stddev_arrow, _, _) = stddev_arrow(&input_arrow, opt_period).unwrap();

        assert_eq!(stddev_arrow.len(), input_close.len());

        let mut out_stddev = vec![0.0; input_close.len()];
        let mut out_sum = vec![0.0; input_close.len()];
        let mut out_sum_sq = vec![0.0; input_close.len()];

        stddev(
            &input_close,
            opt_period,
            &mut out_stddev,
            &mut out_sum,
            &mut out_sum_sq,
        )
        .unwrap();

        for i in 0..input_close.len() {
            if i < 13 {
                #[cfg(feature = "allow-nan")]
                assert!(stddev_arrow.value(i).is_nan());
            } else {
                assert_relative_eq!(stddev_arrow.value(i), out_stddev[i], epsilon = 0.0001);
            }
        }
    }
}
