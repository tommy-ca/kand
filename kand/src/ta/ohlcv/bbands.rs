use crate::{
    KandError, TAFloat,
    ta::{ohlcv::sma, stats::var},
};

#[cfg(feature = "arrow")]
use crate::ta::types::TAArrowArray;

/// Returns the lookback period required for Bollinger Bands calculation.
///
/// # Description
/// The lookback period represents the minimum number of data points needed before
/// the first valid output can be calculated. For Bollinger Bands, this equals
/// the specified period parameter.
///
/// # Arguments
/// * `opt_period` - The time period used for calculations (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period on success, or error on failure
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
///
/// # Example
/// ```
/// use kand::ta::ohlcv::bbands;
/// let period = 20;
/// let lookback = bbands::lookback(period).unwrap();
/// assert_eq!(lookback, 19);
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    sma::lookback(opt_period)
}

/// Calculates Bollinger Bands without input validation for high performance.
pub fn bbands_raw(
    input_price: &[TAFloat],
    opt_period: usize,
    opt_dev_up: TAFloat,
    opt_dev_down: TAFloat,
    output_upper: &mut [TAFloat],
    output_middle: &mut [TAFloat],
    output_lower: &mut [TAFloat],
    output_sma: &mut [TAFloat],
    output_var: &mut [TAFloat],
    output_sum: &mut [TAFloat],
    output_sum_sq: &mut [TAFloat],
) {
    let len = input_price.len();
    let lookback = opt_period - 1;

    // Calculate SMA first
    sma::sma_raw(input_price, opt_period, output_sma);

    // Calculate variance
    var::var_raw(
        input_price,
        opt_period,
        output_var,
        output_sum,
        output_sum_sq,
    );

    for i in lookback..len {
        output_middle[i] = output_sma[i];
        let std_dev = output_var[i].sqrt();

        // Calculate upper and lower bands using standard deviations
        output_upper[i] = opt_dev_up.mul_add(std_dev, output_sma[i]);
        output_lower[i] = opt_dev_down.mul_add(-std_dev, output_sma[i]);
    }
}

/// Calculates Bollinger Bands for a price series.
///
/// # Description
/// Bollinger Bands are volatility bands placed above and below a moving average.
/// They consist of:
/// - A middle band (N-period simple moving average)
/// - An upper band (K standard deviations above middle band)
/// - A lower band (K standard deviations below middle band)
///
/// # Mathematical Formula
/// ```text
/// Middle Band = SMA(price, N)
/// Standard Deviation = sqrt(sum((price - SMA)^2) / N)
/// Upper Band = Middle Band + (K × Standard Deviation)
/// Lower Band = Middle Band - (K × Standard Deviation)
/// ```
/// where:
/// - N is the period
/// - K is the number of standard deviations
///
/// # Calculation Steps
/// 1. Calculate N-period SMA as middle band
/// 2. Calculate N-period standard deviation
/// 3. Add/subtract K standard deviations to get upper/lower bands
///
/// # Arguments
/// * `input_price` - Slice of input price values
/// * `opt_period` - The time period for calculations (must be >= 2)
/// * `opt_dev_up` - Number of standard deviations for upper band
/// * `opt_dev_down` - Number of standard deviations for lower band
/// * `output_upper` - Buffer to store upper band values
/// * `output_middle` - Buffer to store middle band values
/// * `output_lower` - Buffer to store lower band values
/// * `output_sma` - Buffer to store SMA values
/// * `output_var` - Buffer to store variance values
/// * `output_sum` - Buffer to store running sum values
/// * `output_sum_sq` - Buffer to store running sum of squares values
///
/// # Returns
/// * `Result<(), KandError>` - Empty result on success, or error on failure
///
/// # Errors
/// * `KandError::InvalidData` - If input slice is empty
/// * `KandError::LengthMismatch` - If input and output slices have different lengths
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::InsufficientData` - If input length is less than required period
/// * `KandError::NaNDetected` - If any input contains NaN values
///
/// # Example
/// ```
/// use kand::ta::ohlcv::bbands;
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let period = 3;
/// let mut upper = vec![0.0; 5];
/// let mut middle = vec![0.0; 5];
/// let mut lower = vec![0.0; 5];
/// let mut sma = vec![0.0; 5];
/// let mut var = vec![0.0; 5];
/// let mut sum = vec![0.0; 5];
/// let mut sum_sq = vec![0.0; 5];
///
/// bbands::bbands(
///     &prices,
///     period,
///     2.0,
///     2.0,
///     &mut upper,
///     &mut middle,
///     &mut lower,
///     &mut sma,
///     &mut var,
///     &mut sum,
///     &mut sum_sq,
/// )
/// .unwrap();
/// ```
pub fn bbands(
    input_price: &[TAFloat],
    opt_period: usize,
    opt_dev_up: TAFloat,
    opt_dev_down: TAFloat,
    output_upper: &mut [TAFloat],
    output_middle: &mut [TAFloat],
    output_lower: &mut [TAFloat],
    output_sma: &mut [TAFloat],
    output_var: &mut [TAFloat],
    output_sum: &mut [TAFloat],
    output_sum_sq: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_price.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        // Data sufficiency check
        if len == 0 {
            return Err(KandError::InvalidData);
        }

        // Data sufficiency check
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }

        // Length check
        if len != output_upper.len()
            || len != output_middle.len()
            || len != output_lower.len()
            || len != output_sma.len()
            || len != output_var.len()
            || len != output_sum.len()
            || len != output_sum_sq.len()
        {
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

    bbands_raw(
        input_price,
        opt_period,
        opt_dev_up,
        opt_dev_down,
        output_upper,
        output_middle,
        output_lower,
        output_sma,
        output_var,
        output_sum,
        output_sum_sq,
    );

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for i in 0..lookback {
            output_upper[i] = TAFloat::NAN;
            output_middle[i] = TAFloat::NAN;
            output_lower[i] = TAFloat::NAN;
            output_sma[i] = TAFloat::NAN;
            output_var[i] = TAFloat::NAN;
            output_sum[i] = TAFloat::NAN;
            output_sum_sq[i] = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Calculates the next Bollinger Bands values incrementally without validation.
#[must_use]
pub fn bbands_inc_raw(
    input_price: TAFloat,
    prev_sma: TAFloat,
    prev_sum: TAFloat,
    prev_sum_sq: TAFloat,
    input_old_price: TAFloat,
    opt_period: usize,
    opt_dev_up: TAFloat,
    opt_dev_down: TAFloat,
) -> (TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat) {
    let new_sma = sma::sma_inc_raw(input_price, input_old_price, prev_sma, opt_period);
    let (new_variance, new_sum, new_sum_sq) =
        var::var_inc_raw(input_price, prev_sum, prev_sum_sq, input_old_price, opt_period);

    let std_dev = new_variance.sqrt();
    let upper = opt_dev_up.mul_add(std_dev, new_sma);
    let lower = opt_dev_down.mul_add(-std_dev, new_sma);

    (upper, new_sma, lower, new_sma, new_sum, new_sum_sq)
}

/// Calculates the next Bollinger Bands values using an incremental approach.
///
/// # Description
/// This function provides an optimized way to calculate the next set of Bollinger Bands values
/// when new data arrives, without recalculating the entire series. It uses the previous values
/// to compute the new bands efficiently.
///
/// # Calculation Steps
/// 1. Calculate new SMA using incremental approach
/// 2. Calculate new variance using incremental approach
/// 3. Compute standard deviation and bands
///
/// # Arguments
/// * `input_price` - The current price value
/// * `prev_sma` - The previous SMA value
/// * `prev_sum` - The previous sum for variance calculation
/// * `prev_sum_sq` - The previous sum of squares for variance calculation
/// * `input_old_price` - The oldest price value to be removed from the period
/// * `opt_period` - The time period for calculations (must be >= 2)
/// * `opt_dev_up` - Number of standard deviations for upper band
/// * `opt_dev_down` - Number of standard deviations for lower band
///
/// # Returns
/// * `Result<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat), KandError>` - A tuple containing:
///   - Upper Band value
///   - Middle Band value
///   - Lower Band value
///   - New SMA value
///   - New Sum value
///   - New Sum of Squares value
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::NaNDetected` - If any input contains NaN values
///
/// # Example
/// ```
/// use kand::ta::ohlcv::bbands;
/// let (upper, middle, lower, sma, sum, sum_sq) = bbands::bbands_inc(
///     10.0,   // new price
///     9.5,    // previous SMA
///     28.5,   // previous sum
///     272.25, // previous sum of squares
///     9.0,    // oldest price
///     3,      // period
///     2.0,    // upper deviation
///     2.0,    // lower deviation
/// )
/// .unwrap();
/// ```
pub fn bbands_inc(
    input_price: TAFloat,
    prev_sma: TAFloat,
    prev_sum: TAFloat,
    prev_sum_sq: TAFloat,
    input_old_price: TAFloat,
    opt_period: usize,
    opt_dev_up: TAFloat,
    opt_dev_down: TAFloat,
) -> Result<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat, TAFloat), KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        if input_price.is_nan()
            || prev_sma.is_nan()
            || prev_sum.is_nan()
            || prev_sum_sq.is_nan()
            || input_old_price.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
        if opt_dev_up.is_nan() || opt_dev_down.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(bbands_inc_raw(
        input_price,
        prev_sma,
        prev_sum,
        prev_sum_sq,
        input_old_price,
        opt_period,
        opt_dev_up,
        opt_dev_down,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper_multi!(
    bbands,
    crate::ta::ohlcv::bbands::bbands_raw,
    inputs: { input_price },
    params: {
        opt_period: usize,
        opt_dev_up: TAFloat,
        opt_dev_down: TAFloat
    },
    outputs: {
        output_upper,
        output_middle,
        output_lower,
        output_sma,
        output_var,
        output_sum,
        output_sum_sq
    }
);

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_bbands_calculation() {
        let input_price = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4, 35172.0, 35176.7, 35113.3, 35114.7, 35129.3,
        ];

        let opt_period = 20;
        let opt_dev_up = 2.0;
        let opt_dev_down = 2.0;
        let mut output_upper = vec![0.0; input_price.len()];
        let mut output_middle = vec![0.0; input_price.len()];
        let mut output_lower = vec![0.0; input_price.len()];
        let mut output_sma = vec![0.0; input_price.len()];
        let mut output_var = vec![0.0; input_price.len()];
        let mut output_sum = vec![0.0; input_price.len()];
        let mut output_sum_sq = vec![0.0; input_price.len()];

        bbands(
            &input_price,
            opt_period,
            opt_dev_up,
            opt_dev_down,
            &mut output_upper,
            &mut output_middle,
            &mut output_lower,
            &mut output_sma,
            &mut output_var,
            &mut output_sum,
            &mut output_sum_sq,
        )
        .unwrap();

        // First 19 values should be NaN
        #[cfg(feature = "allow-nan")]
        for i in 0..19 {
            assert!(output_upper[i].is_nan());
            assert!(output_middle[i].is_nan());
            assert!(output_lower[i].is_nan());
            assert!(output_sma[i].is_nan());
            assert!(output_var[i].is_nan());
            assert!(output_sum[i].is_nan());
            assert!(output_sum_sq[i].is_nan());
        }

        // Compare with known values
        let expected_upper = vec![
            35_315.492_158_169_03,
            35_324.023_520_348_93,
            35_323.822_186_479_93,
            35_319.449_647_081_79,
            35_314.110_592_229_015,
            35_306.809_201_120_76,
            35_288.014_966_586_7,
            35_276.648_971_890_07,
            35_253.671_769_987_47,
            35_239.448_423_376_95,
            35_232.360_028_616_255,
            35_221.822_196_455_76,
            35_200.867_114_660_425,
            35_179.557_912_368_81,
            35_172.678_349_978_625,
            35_185.898_169_265_74,
            35_210.535_957_882_84,
            35_218.090_674_365_98,
            35_236.434_486_030_11,
            35_252.252_647_217_1,
            35_257.112_658_379_57,
            35_250.714_615_459_73,
            35_240.881_227_372_27,
            35_230.468_530_636_03,
            35_225.037_992_782_84,
            35_223.587_496_067_295,
        ];

        for i in 0..expected_upper.len() {
            assert_relative_eq!(output_upper[i + 19], expected_upper[i], epsilon = 0.0001);
        }

        // Test incremental calculation
        let mut prev_sma = output_sma[19];
        let mut prev_sum = output_sum[19];
        let mut prev_sum_sq = output_sum_sq[19];

        for i in 20..45 {
            let (upper, middle, lower, new_sma, new_sum, new_sum_sq) = bbands_inc(
                input_price[i],
                prev_sma,
                prev_sum,
                prev_sum_sq,
                input_price[i - opt_period],
                opt_period,
                opt_dev_up,
                opt_dev_down,
            )
            .unwrap();

            assert_relative_eq!(upper, output_upper[i], epsilon = 0.0001);
            assert_relative_eq!(middle, output_middle[i], epsilon = 0.0001);
            assert_relative_eq!(lower, output_lower[i], epsilon = 0.0001);

            prev_sma = new_sma;
            prev_sum = new_sum;
            prev_sum_sq = new_sum_sq;
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_bbands_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_price = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4, 35172.0, 35176.7, 35113.3, 35114.7, 35129.3,
        ];

        let input_arrow = TAArrowArray::from(input_price.clone());
        let opt_period = 20;
        let opt_dev_up = 2.0;
        let opt_dev_down = 2.0;

        let (upper, middle, lower, _, _, _, _) =
            bbands_arrow(&input_arrow, opt_period, opt_dev_up, opt_dev_down).unwrap();

        assert_eq!(upper.len(), input_price.len());

        let mut out_upper = vec![0.0; input_price.len()];
        let mut out_middle = vec![0.0; input_price.len()];
        let mut out_lower = vec![0.0; input_price.len()];
        let mut out_sma = vec![0.0; input_price.len()];
        let mut out_var = vec![0.0; input_price.len()];
        let mut out_sum = vec![0.0; input_price.len()];
        let mut out_sum_sq = vec![0.0; input_price.len()];

        bbands(
            &input_price,
            opt_period,
            opt_dev_up,
            opt_dev_down,
            &mut out_upper,
            &mut out_middle,
            &mut out_lower,
            &mut out_sma,
            &mut out_var,
            &mut out_sum,
            &mut out_sum_sq,
        )
        .unwrap();

        for i in 0..input_price.len() {
            if i < 19 {
                #[cfg(feature = "allow-nan")]
                {
                    assert!(upper.value(i).is_nan());
                    assert!(middle.value(i).is_nan());
                    assert!(lower.value(i).is_nan());
                }
            } else {
                assert_relative_eq!(upper.value(i), out_upper[i], epsilon = 0.0001);
                assert_relative_eq!(middle.value(i), out_middle[i], epsilon = 0.0001);
                assert_relative_eq!(lower.value(i), out_lower[i], epsilon = 0.0001);
            }
        }
    }
}
