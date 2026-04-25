use super::adx;
use crate::{KandError, TAFloat};

/// Calculates the lookback period required for ADXR calculation
///
/// # Arguments
/// * `opt_period` - The period parameter for ADX calculation (typically 14)
///
/// # Returns
/// * `Result<usize, KandError>` - The number of data points needed before first valid ADXR value
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
///
/// # Example
/// ```
/// use kand::ohlcv::adxr::lookback;
/// let period = 14;
/// let lookback_period = lookback(period).unwrap();
/// assert_eq!(lookback_period, 40); // 14 * 3 - 2 = 40
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period * 3 - 2)
}

/// Calculates ADXR without input validation for high performance.
pub fn adxr_raw(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_period: usize,
    output_adxr: &mut [TAFloat],
) {
    let len = input_high.len();
    let lookback = opt_period * 3 - 2;

    let mut output_adx = vec![0.0; len];
    let mut output_smoothed_plus_dm = vec![0.0; len];
    let mut output_smoothed_minus_dm = vec![0.0; len];
    let mut output_smoothed_tr = vec![0.0; len];

    // Calculate ADX first
    adx::adx_raw(
        input_high,
        input_low,
        input_close,
        opt_period,
        &mut output_adx,
        &mut output_smoothed_plus_dm,
        &mut output_smoothed_minus_dm,
        &mut output_smoothed_tr,
    );

    // Calculate ADXR = (Current ADX + ADX period-1 periods ago) / 2
    for i in lookback..len {
        output_adxr[i] = (output_adx[i] + output_adx[i - opt_period + 1]) / 2.0;
    }
}

/// Calculates the Average Directional Index Rating (ADXR) for the entire input array
///
/// # Mathematical Formula
/// ```text
/// ADXR = (Current ADX + ADX period days ago) / 2
/// ```
///
/// # Calculation Principle
/// 1. Calculate ADX for the entire dataset
/// 2. For each point starting from position (3*period-2):
///    - Take current ADX value
///    - Take ADX value from `period` days ago
///    - Calculate average of these two values
///
/// # Arguments
/// * `input_high` - Array of high prices
/// * `input_low` - Array of low prices
/// * `input_close` - Array of closing prices
/// * `opt_period` - Period for ADX calculation
/// * `output_adxr` - Output array for ADXR values
///
/// # Returns
/// * `Result<(), KandError>` - Ok if calculation succeeds
///
/// # Errors
/// * `KandError::InvalidData` - If input arrays are empty
/// * `KandError::LengthMismatch` - If input/output arrays have different lengths
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::InsufficientData` - If input length is less than required lookback period
/// * `KandError::NaNDetected` - If any input value is NaN (when "`check-nan`" feature enabled)
///
/// # Example
/// ```
/// use kand::ohlcv::adxr::adxr;
///
/// let input_high = vec![24.20, 24.07, 24.04, 23.87, 23.67];
/// let input_low = vec![23.85, 23.72, 23.64, 23.37, 23.46];
/// let input_close = vec![23.89, 23.95, 23.67, 23.78, 23.50];
/// let period = 2;
/// let mut output_adxr = vec![0.0; 5];
///
/// adxr(
///     &input_high,
///     &input_low,
///     &input_close,
///     period,
///     &mut output_adxr,
/// )
/// .unwrap();
/// ```
pub fn adxr(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_period: usize,
    output_adxr: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_high.len();
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
        if len != input_low.len() || len != input_close.len() || len != output_adxr.len() {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for i in 0..len {
            // NaN check
            if input_high[i].is_nan() || input_low[i].is_nan() || input_close[i].is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    adxr_raw(input_high, input_low, input_close, opt_period, output_adxr);

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for i in 0..lookback {
            output_adxr[i] = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Calculates the latest ADXR value incrementally without validation
pub fn adxr_inc_raw(
    input_high: TAFloat,
    input_low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    prev_close: TAFloat,
    prev_adx: TAFloat,
    prev_adx_period_ago: TAFloat,
    prev_smoothed_plus_dm: TAFloat,
    prev_smoothed_minus_dm: TAFloat,
    prev_smoothed_tr: TAFloat,
    opt_period: usize,
) -> (TAFloat, TAFloat, TAFloat, TAFloat, TAFloat) {
    let (output_adx, output_smoothed_plus_dm, output_smoothed_minus_dm, output_smoothed_tr) =
        adx::adx_inc_raw(
            input_high,
            input_low,
            prev_high,
            prev_low,
            prev_close,
            prev_adx,
            prev_smoothed_plus_dm,
            prev_smoothed_minus_dm,
            prev_smoothed_tr,
            opt_period,
        );

    let output_adxr = (output_adx + prev_adx_period_ago) / 2.0;

    (
        output_adxr,
        output_adx,
        output_smoothed_plus_dm,
        output_smoothed_minus_dm,
        output_smoothed_tr,
    )
}

/// Calculates the latest ADXR value incrementally
///
/// # Mathematical Formula
/// ```text
/// Latest ADXR = (Latest ADX + ADX period days ago) / 2
/// ```
///
/// # Arguments
/// * `input_high` - Current high price
/// * `input_low` - Current low price
/// * `prev_high` - Previous high price
/// * `prev_low` - Previous low price
/// * `prev_close` - Previous close price
/// * `prev_adx` - Previous ADX value
/// * `prev_adx_period_ago` - ADX value from period days ago
/// * `prev_smoothed_plus_dm` - Previous smoothed +DM value
/// * `prev_smoothed_minus_dm` - Previous smoothed -DM value
/// * `prev_smoothed_tr` - Previous smoothed TR value
/// * `opt_period` - Period for ADX calculation
///
/// # Returns
/// * `Result<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat), KandError>` - Tuple containing:
///   - Latest ADXR value
///   - Latest ADX value
///   - New smoothed +DM
///   - New smoothed -DM
///   - New smoothed TR
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::NaNDetected` - If any input value is NaN (when "`check-nan`" feature enabled)
///
/// # Example
/// ```
/// use kand::ohlcv::adxr::adxr_inc;
///
/// let (adxr, adx, plus_dm, minus_dm, tr) = adxr_inc(
///     24.20, // input_high
///     23.85, // input_low
///     24.07, // prev_high
///     23.72, // prev_low
///     23.95, // prev_close
///     25.0,  // prev_adx
///     20.0,  // prev_adx_period_ago
///     0.5,   // prev_smoothed_plus_dm
///     0.3,   // prev_smoothed_minus_dm
///     1.2,   // prev_smoothed_tr
///     14,    // opt_period
/// )
/// .unwrap();
/// ```
pub fn adxr_inc(
    input_high: TAFloat,
    input_low: TAFloat,
    prev_high: TAFloat,
    prev_low: TAFloat,
    prev_close: TAFloat,
    prev_adx: TAFloat,
    prev_adx_period_ago: TAFloat,
    prev_smoothed_plus_dm: TAFloat,
    prev_smoothed_minus_dm: TAFloat,
    prev_smoothed_tr: TAFloat,
    opt_period: usize,
) -> Result<(TAFloat, TAFloat, TAFloat, TAFloat, TAFloat), KandError> {
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
        if input_high.is_nan()
            || input_low.is_nan()
            || prev_high.is_nan()
            || prev_low.is_nan()
            || prev_close.is_nan()
            || prev_adx.is_nan()
            || prev_adx_period_ago.is_nan()
            || prev_smoothed_plus_dm.is_nan()
            || prev_smoothed_minus_dm.is_nan()
            || prev_smoothed_tr.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(adxr_inc_raw(
        input_high,
        input_low,
        prev_high,
        prev_low,
        prev_close,
        prev_adx,
        prev_adx_period_ago,
        prev_smoothed_plus_dm,
        prev_smoothed_minus_dm,
        prev_smoothed_tr,
        opt_period,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    adxr_arrow,
    crate::ta::ohlcv::adxr::adxr_raw,
    inputs: { input_high, input_low, input_close },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use arrow::array::Array;
    use approx::assert_relative_eq;

    use super::*;

    // Basic functionality tests
    #[test]
    fn test_adxr_calculation() {
        let input_high = vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35010.8, 35047.1, 35091.4, 35150.4, 35123.9,
            35110.0, 35092.1, 35179.2, 35244.9, 35150.2, 35136.0, 35133.6, 35188.0, 35215.3,
            35221.9, 35219.2, 35234.0, 35216.7, 35197.9, 35178.4, 35183.4, 35129.7, 35149.1,
            35129.3, 35125.5, 35114.5, 35120.1, 35129.4, 35105.4, 35054.1, 35034.6, 35032.9,
            35070.8, 35086.0, 35086.9, 35048.9, 34988.6, 35004.3,
        ];
        let input_low = vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35203.5, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34911.0, 34952.5, 34977.9, 35039.0, 35073.0, 35055.0,
            35084.0, 35060.0, 35073.1, 35090.0, 35072.0, 35078.0, 35088.0, 35124.8, 35169.4,
            35138.0, 35141.0, 35182.0, 35151.1, 35158.4, 35140.0, 35087.0, 35085.8, 35114.7,
            35086.0, 35090.6, 35074.1, 35078.4, 35100.0, 35030.2, 34986.3, 34988.1, 34973.1,
            35012.3, 35048.3, 35038.9, 34937.3, 34937.0, 34958.7,
        ];
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4, 35172.0, 35176.7, 35113.3, 35114.7, 35129.3,
            35094.6, 35114.4, 35094.5, 35116.0, 35105.4, 35050.7, 35031.3, 35008.1, 35021.4,
            35048.4, 35080.1, 35043.6, 34962.7, 34970.1, 34980.1,
        ];
        let opt_period = 14;
        let mut output_adxr = vec![0.0; input_high.len()];

        adxr(
            &input_high,
            &input_low,
            &input_close,
            opt_period,
            &mut output_adxr,
        )
        .unwrap();

        // First (3*period-2) values should be NaN
        #[cfg(feature = "allow-nan")]
        for value in output_adxr.iter().take(3 * opt_period - 2) {
            assert!(value.is_nan());
        }

        // Test specific values from the dataset
        let expected_values = [
            22.811_892_152_364_344,
            21.796_737_720_532_633,
            20.830_226_119_467_59,
            20.335_909_265_463_94,
            19.633_114_317_983_484,
            19.152_494_831_686_187,
            18.706_205_308_695_843,
            18.801_762_117_450_48,
            18.972_687_680_470_624,
            18.678_498_730_567_725,
            19.081_591_663_045_668,
            19.877_741_529_748_13,
            20.271_853_926_105_9,
            20.745_128_023_359_108,
            20.439_791_113_055_39,
            19.688_560_813_255_584,
            19.089_803_821_272_966,
            19.186_732_053_592_11,
            19.450_462_541_153_087,
            19.478_837_411_178_546,
        ];

        let first_valid_idx = 3 * opt_period - 2;
        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_adxr[i + first_valid_idx], *expected, epsilon = 0.05);
        }

        // To test incremental, we need ADX and DM/TR values.
        // We'll calculate them manually for the test.
        let mut out_adx = vec![0.0; input_high.len()];
        let mut out_plus_dm = vec![0.0; input_high.len()];
        let mut out_minus_dm = vec![0.0; input_high.len()];
        let mut out_tr = vec![0.0; input_high.len()];
        adx::adx_raw(
            &input_high,
            &input_low,
            &input_close,
            opt_period,
            &mut out_adx,
            &mut out_plus_dm,
            &mut out_minus_dm,
            &mut out_tr,
        );

        // Calculate and verify incremental values starting from index period * 4 - 3
        for i in (opt_period * 4 - 3)..input_high.len() {
            let result = adxr_inc(
                input_high[i],
                input_low[i],
                input_high[i - 1],
                input_low[i - 1],
                input_close[i - 1],
                out_adx[i - 1],
                out_adx[i - opt_period + 1], // ADX value from period-1 days ago
                out_plus_dm[i - 1],
                out_minus_dm[i - 1],
                out_tr[i - 1],
                opt_period,
            )
            .unwrap();

            // Compare with full calculation
            assert_relative_eq!(result.0, output_adxr[i], epsilon = 1.0); // ADXR value
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_adxr_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_high = vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4, 35172.0, 35176.7, 35113.3, 35114.7, 35129.3,
            35094.6, 35114.4, 35094.5, 35116.0, 35105.4, 35050.7, 35031.3, 35008.1, 35021.4,
            35048.4, 35080.1, 35043.6, 34962.7, 34970.1, 34980.1,
        ];
        let input_low = vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35203.5, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34931.0, 34952.5, 34977.9, 35039.0, 35073.0, 35055.0,
            35084.0, 35060.0, 35073.1, 35090.0, 35072.0, 35078.0, 35088.0, 35124.8, 35169.4,
            35138.0, 35141.0, 35182.0, 35151.1, 35158.4, 35140.0, 35087.0, 35085.8, 35114.7,
            35086.0, 35090.6, 35074.1, 35078.4, 35100.0, 35030.2, 34986.3, 34988.1, 34973.1,
            35012.3, 35048.3, 35038.9, 34937.3, 34937.0, 34958.7,
        ];
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4, 35172.0, 35176.7, 35113.3, 35114.7, 35129.3,
            35094.6, 35114.4, 35094.5, 35116.0, 35105.4, 35050.7, 35031.3, 35008.1, 35021.4,
            35048.4, 35080.1, 35043.6, 34962.7, 34970.1, 34980.1,
        ];

        let high_arrow = TAArrowArray::from(input_high.clone());
        let low_arrow = TAArrowArray::from(input_low.clone());
        let close_arrow = TAArrowArray::from(input_close.clone());
        let opt_period = 14;

        let adxr_arrow = adxr_arrow(&high_arrow, &low_arrow, &close_arrow, opt_period).unwrap();

        assert_eq!(adxr_arrow.len(), input_high.len());

        let mut out_adxr = vec![0.0; input_high.len()];

        adxr(
            &input_high,
            &input_low,
            &input_close,
            opt_period,
            &mut out_adxr,
        )
        .unwrap();

        for i in 0..input_high.len() {
            if i < 40 {
                #[cfg(feature = "allow-nan")]
                assert!(adxr_arrow.value(i).is_nan());
            } else {
                assert_relative_eq!(adxr_arrow.value(i), out_adxr[i], epsilon = 0.05);
            }
        }
    }
}
