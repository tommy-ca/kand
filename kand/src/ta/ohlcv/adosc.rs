use super::{ad, ema};
use crate::{KandError, TAFloat, TAPeriod, ta::types::MAType};


/// Returns the lookback period for ADOSC without input validation.
#[inline]
#[must_use]
pub const fn lookback_raw(
    _opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    _opt_ma_type: MAType,
) -> TAPeriod {
    opt_slow_period - 1
}

/// Returns the lookback period required for A/D Oscillator calculation.
///
/// The A/D Oscillator requires a lookback equal to the slow EMA period minus one.
///
/// # Errors
///
/// - [`KandError::InvalidParameter`] if fast or slow period is less than 2, or if fast period is not less than slow period (enabled by "check" feature).
///
/// # Examples
///
/// ```
/// use kand::ohlcv::adosc;
/// use kand::ta::types::MAType;
/// let lookback = adosc::lookback(3, 10, MAType::EMA).unwrap();
/// assert_eq!(lookback, 9);
/// ```
#[must_use]
pub const fn lookback(
    opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    opt_ma_type: MAType,
) -> Result<TAPeriod, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_fast_period < 2 || opt_slow_period < 2 || opt_fast_period >= opt_slow_period {
            return Err(KandError::InvalidParameter);
        }
    }

    Ok(lookback_raw(opt_fast_period, opt_slow_period, opt_ma_type))
}

/// Computes ADOSC without input validation for high performance.
pub fn adosc_raw(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    input_volume: &[TAFloat],
    opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    opt_ma_type: MAType,
    output_adosc: &mut [TAFloat],
) {
    let len = input_high.len();
    let mut output_ad = vec![0.0; len];
    let mut output_ad_fast_ema = vec![0.0; len];
    let mut output_ad_slow_ema = vec![0.0; len];

    ad::ad_raw(input_high, input_low, input_close, input_volume, &mut output_ad);

    ema::ema_raw(&output_ad, opt_fast_period, None, &mut output_ad_fast_ema);
    ema::ema_raw(&output_ad, opt_slow_period, None, &mut output_ad_slow_ema);

    let lookback = lookback_raw(opt_fast_period, opt_slow_period, opt_ma_type);
    for i in lookback..len {
        output_adosc[i] = output_ad_fast_ema[i] - output_ad_slow_ema[i];
    }
}

/// Calculates the Accumulation/Distribution Oscillator (A/D Oscillator or ADOSC) for the entire price series.
///
/// The A/D Oscillator is the difference between fast and slow EMAs of the Accumulation/Distribution (A/D) line.
/// It helps identify trend strength and potential reversals by measuring momentum in money flow.
///
/// # Formula
///
/// ```text
/// Money Flow Multiplier (MFM) = ((Close - Low) - (High - Close)) / (High - Low)
/// Money Flow Volume (MFV) = MFM * Volume
/// A/D = Previous A/D + MFV
/// ADOSC = EMA(A/D, fast_period) - EMA(A/D, slow_period)
/// ```
///
/// # Calculation
///
/// 1. Compute the A/D line as the cumulative sum of MFV (see A/D documentation for details).
/// 2. Calculate the fast EMA of the A/D line.
/// 3. Calculate the slow EMA of the A/D line.
/// 4. Subtract the slow EMA from the fast EMA to get ADOSC.
///
/// If High - Low is zero, MFM is set to 0 to avoid division by zero.
/// Outputs for the first `lookback` periods are set to NaN.
///
/// # Errors
///
/// - [`KandError::InvalidData`] if input arrays are empty (enabled by "check" feature).
/// - [`KandError::InsufficientData`] if input length is less than or equal to lookback (enabled by "check" feature).
/// - [`KandError::LengthMismatch`] if input or output arrays have different lengths (enabled by "check" feature).
/// - [`KandError::InvalidParameter`] if periods are invalid (propagated from lookback).
/// - [`KandError::NaNDetected`] if any input contains NaN values (enabled by "check-nan" feature).
///
/// # Examples
///
/// ```
/// use kand::ohlcv::adosc;
/// use kand::ta::types::MAType;
/// let high = vec![10.0, 11.0, 12.0, 11.5, 10.5];
/// let low = vec![8.0, 9.0, 10.0, 9.5, 8.5];
/// let close = vec![9.0, 10.0, 11.0, 10.0, 9.0];
/// let volume = vec![100.0, 150.0, 200.0, 150.0, 100.0];
/// let mut adosc_out = vec![0.0; 5];
///
/// adosc::adosc(
///     &high,
///     &low,
///     &close,
///     &volume,
///     3,
///     5,
///     MAType::EMA,
///     &mut adosc_out,
/// )
/// .unwrap();
/// ```
pub fn adosc(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    input_volume: &[TAFloat],
    opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    opt_ma_type: MAType,
    output_adosc: &mut [TAFloat],
) -> Result<(), KandError> {
    let len = input_high.len();
    let lookback = lookback(opt_fast_period, opt_slow_period, opt_ma_type)?;

    #[cfg(feature = "check")]
    {
        if len == 0 {
            return Err(KandError::InvalidData);
        }

        if len <= lookback {
            return Err(KandError::InsufficientData);
        }

        if len != input_low.len()
            || len != input_close.len()
            || len != input_volume.len()
            || len != output_adosc.len()
        {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for i in 0..len {
            if input_high[i].is_nan()
                || input_low[i].is_nan()
                || input_close[i].is_nan()
                || input_volume[i].is_nan()
            {
                return Err(KandError::NaNDetected);
            }
        }
    }

    adosc_raw(
        input_high,
        input_low,
        input_close,
        input_volume,
        opt_fast_period,
        opt_slow_period,
        opt_ma_type,
        output_adosc,
    );

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for i in 0..lookback {
            output_adosc[i] = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Computes the next ADOSC values incrementally without input validation.
#[inline]
#[must_use]
pub fn adosc_inc_raw(
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    input_volume: TAFloat,
    prev_ad: TAFloat,
    prev_ad_fast_ema: TAFloat,
    prev_ad_slow_ema: TAFloat,
    fast_multiplier: TAFloat,
    slow_multiplier: TAFloat,
) -> (TAFloat, TAFloat, TAFloat, TAFloat) {
    let output_ad = ad::ad_inc_raw(input_high, input_low, input_close, input_volume, prev_ad);
    let output_ad_fast_ema = ema::ema_inc_raw(output_ad, prev_ad_fast_ema, fast_multiplier);
    let output_ad_slow_ema = ema::ema_inc_raw(output_ad, prev_ad_slow_ema, slow_multiplier);
    let output_adosc = output_ad_fast_ema - output_ad_slow_ema;

    (
        output_adosc,
        output_ad,
        output_ad_fast_ema,
        output_ad_slow_ema,
    )
}

/// Calculates the latest A/D Oscillator value incrementally using previous values.
///
/// This is an optimized version that computes only the latest ADOSC value, avoiding recalculation of the entire series.
///
/// # Formula
///
/// ```text
/// Money Flow Multiplier (MFM) = ((Close - Low) - (High - Close)) / (High - Low)
/// Money Flow Volume (MFV) = MFM * Volume
/// Latest A/D = Previous A/D + MFV
/// Latest Fast EMA = (Latest A/D - Previous Fast EMA) * (2 / (fast_period + 1)) + Previous Fast EMA
/// Latest Slow EMA = (Latest A/D - Previous Slow EMA) * (2 / (slow_period + 1)) + Previous Slow EMA
/// Latest ADOSC = Latest Fast EMA - Latest Slow EMA
/// ```
///
/// If High - Low is zero, MFM is set to 0 to avoid division by zero.
///
/// # Errors
///
/// - [`KandError::InvalidParameter`] if fast or slow period is 0, or if fast period is not less than slow period (enabled by "check" feature).
/// - [`KandError::NaNDetected`] if any input contains NaN values (enabled by "check-nan" feature).
///
/// # Examples
///
/// ```
/// use kand::ohlcv::adosc;
/// use kand::ta::types::MAType;
/// let (adosc, ad, ad_fast_ema, ad_slow_ema) = adosc::adosc_inc(
///     10.5,
///     9.5,
///     10.0,
///     150.0,
///     100.0,
///     95.0,
///     90.0,
///     3,
///     10,
///     MAType::EMA,
/// )
/// .unwrap();
/// ```
pub fn adosc_inc(
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    input_volume: TAFloat,
    prev_ad: TAFloat,
    prev_ad_fast_ema: TAFloat,
    prev_ad_slow_ema: TAFloat,
    opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    _opt_ma_type: MAType,
) -> Result<(TAFloat, TAFloat, TAFloat, TAFloat), KandError> {
    #[cfg(feature = "check")]
    {
        if opt_fast_period < 2 || opt_slow_period < 2 || opt_fast_period >= opt_slow_period {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        if input_high.is_nan()
            || input_low.is_nan()
            || input_close.is_nan()
            || input_volume.is_nan()
            || prev_ad.is_nan()
            || prev_ad_fast_ema.is_nan()
            || prev_ad_slow_ema.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    let fast_multiplier = 2.0 / (opt_fast_period + 1) as TAFloat;
    let slow_multiplier = 2.0 / (opt_slow_period + 1) as TAFloat;

    Ok(adosc_inc_raw(
        input_high,
        input_low,
        input_close,
        input_volume,
        prev_ad,
        prev_ad_fast_ema,
        prev_ad_slow_ema,
        fast_multiplier,
        slow_multiplier,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    adosc_arrow,
    crate::ta::ohlcv::adosc::adosc_raw,
    inputs: { input_high, input_low, input_close, input_volume },
    params: { opt_fast_period: usize, opt_slow_period: usize, opt_ma_type: crate::ta::types::MAType },
    lookback_params: { opt_fast_period, opt_slow_period, opt_ma_type }
);

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::EPSILON;

    use super::*;

    /// Tests the calculation of A/D Oscillator for a full series and verifies incremental calculations match.
    #[test]
    fn test_adosc_calculation() {
        let input_high = vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3,
        ];
        let input_low = vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35203.5, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34911.0, 34952.5, 34977.9, 35039.0, 35073.0, 35055.0,
            35084.0, 35060.0, 35073.1,
        ];
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3,
        ];
        let input_volume = vec![
            1055.365, 756.488, 682.152, 1197.747, 425.97, 859.638, 741.925, 888.477, 1043.333,
            467.901, 387.47, 566.099, 672.296, 834.915, 1854.024, 3670.795, 3761.198, 1605.442,
            1726.574, 934.713, 2199.061, 2349.823, 837.218, 1000.638, 1218.202, 2573.668, 1098.409,
            609.582, 670.489, 1637.998,
        ];
        let opt_fast_period = 3;
        let opt_slow_period = 10;
        let mut output_adosc = vec![0.0; input_high.len()];

        adosc(
            &input_high,
            &input_low,
            &input_close,
            &input_volume,
            opt_fast_period,
            opt_slow_period,
            MAType::EMA,
            &mut output_adosc,
        )
        .unwrap();

        let expected_values = [
            -20.897_560_400_954_944,
            -113.006_596_430_246_87,
            39.233_539_470_168_466,
            90.168_278_254_455_34,
            -42.550_171_226_589_67,
            -592.067_697_274_955_4,
            -1495.471_434_093_873_8,
            -1719.284_357_755_546_8,
            -1260.209_773_442_224_6,
            -570.395_499_498_773_5,
            -511.021_422_797_917_7,
            -1032.120_377_773_458_1,
            -1051.642_820_428_754_5,
            -796.333_736_224_223_7,
            -349.880_245_462_221_4,
            83.522_227_098_310_85,
            312.478_479_876_569_96,
            456.358_684_600_642_4,
            399.132_419_401_690_64,
            301.554_565_477_041_25,
            363.412_358_422_480_8,
        ];
        for (i, &expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_adosc[i + 9], expected, epsilon = EPSILON);
        }

        // Test incremental
        // To test incremental, we still need the intermediate values because adosc_inc needs them.
        // We'll calculate them manually for the test.
        let mut ad_values = vec![0.0; input_high.len()];
        ad::ad_raw(&input_high, &input_low, &input_close, &input_volume, &mut ad_values);
        let mut ad_fast_ema = vec![0.0; input_high.len()];
        let mut ad_slow_ema = vec![0.0; input_high.len()];
        ema::ema_raw(&ad_values, opt_fast_period, None, &mut ad_fast_ema);
        ema::ema_raw(&ad_values, opt_slow_period, None, &mut ad_slow_ema);

        let mut prev_ad = ad_values[9];
        let mut prev_ad_fast_ema = ad_fast_ema[9];
        let mut prev_ad_slow_ema = ad_slow_ema[9];

        for i in 10..input_high.len() {
            let (output_adosc_inc, output_ad_inc, output_ad_fast_ema_inc, output_ad_slow_ema_inc) =
                adosc_inc(
                    input_high[i],
                    input_low[i],
                    input_close[i],
                    input_volume[i],
                    prev_ad,
                    prev_ad_fast_ema,
                    prev_ad_slow_ema,
                    opt_fast_period,
                    opt_slow_period,
                    MAType::EMA,
                )
                .unwrap();
            assert_relative_eq!(output_adosc_inc, output_adosc[i], epsilon = EPSILON);
            prev_ad = output_ad_inc;
            prev_ad_fast_ema = output_ad_fast_ema_inc;
            prev_ad_slow_ema = output_ad_slow_ema_inc;
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_adosc_arrow() {
        use crate::ta::types::TAArrowArray;

        let high_arrow = TAArrowArray::from(vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3,
        ]);
        let low_arrow = TAArrowArray::from(vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35203.5, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34911.0, 34952.5, 34977.9, 35039.0, 35073.0, 35055.0,
            35084.0, 35060.0, 35073.1,
        ]);
        let close_arrow = TAArrowArray::from(vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3,
        ]);
        let volume_arrow = TAArrowArray::from(vec![
            1055.365, 756.488, 682.152, 1197.747, 425.97, 859.638, 741.925, 888.477, 1043.333,
            467.901, 387.47, 566.099, 672.296, 834.915, 1854.024, 3670.795, 3761.198, 1605.442,
            1726.574, 934.713, 2199.061, 2349.823, 837.218, 1000.638, 1218.202, 2573.668, 1098.409,
            609.582, 670.489, 1637.998,
        ]);
        let opt_fast_period = 3;
        let opt_slow_period = 10;

        let adosc_arrow = adosc_arrow(
            &high_arrow,
            &low_arrow,
            &close_arrow,
            &volume_arrow,
            opt_fast_period,
            opt_slow_period,
            MAType::EMA,
        )
        .unwrap();

        assert_eq!(adosc_arrow.len(), 30);

        let expected_values = [
            -20.897_560_400_954_944,
            -113.006_596_430_246_87,
            39.233_539_470_168_466,
            90.168_278_254_455_34,
            -42.550_171_226_589_67,
            -592.067_697_274_955_4,
            -1495.471_434_093_873_8,
            -1719.284_357_755_546_8,
            -1260.209_773_442_224_6,
            -570.395_499_498_773_5,
            -511.021_422_797_917_7,
            -1032.120_377_773_458_1,
            -1051.642_820_428_754_5,
            -796.333_736_224_223_7,
            -349.880_245_462_221_4,
            83.522_227_098_310_85,
            312.478_479_876_569_96,
            456.358_684_600_642_4,
            399.132_419_401_690_64,
            301.554_565_477_041_25,
            363.412_358_422_480_8,
        ];

        for (i, &expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(adosc_arrow.value(i + 9), expected, epsilon = EPSILON);
        }
    }
}
