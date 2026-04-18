use super::ad;
use crate::{KandError, TAFloat, TAPeriod, ta::types::MAType};

/// Returns the lookback period for ADOSC without input validation.
#[inline]
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
    let mut ad_values = vec![0.0; len];
    ad::ad_raw(
        input_high,
        input_low,
        input_close,
        input_volume,
        &mut ad_values,
    );

    let lookback = lookback_raw(opt_fast_period, opt_slow_period, opt_ma_type);

    let fast_k = 2.0 / (opt_fast_period as TAFloat + 1.0);
    let slow_k = 2.0 / (opt_slow_period as TAFloat + 1.0);

    // Initial SMA for EMAs
    let mut fast_sum = 0.0;
    for ad_val in ad_values.iter().take(opt_fast_period) {
        fast_sum += ad_val;
    }
    let mut fast_ema = fast_sum / opt_fast_period as TAFloat;

    let mut slow_sum = 0.0;
    for ad_val in ad_values.iter().take(opt_slow_period) {
        slow_sum += ad_val;
    }
    let mut slow_ema = slow_sum / opt_slow_period as TAFloat;

    // Fill NaNs
    for adosc_val in output_adosc.iter_mut().take(len) {
        *adosc_val = TAFloat::NAN;
    }

    // First valid ADOSC is at index opt_slow_period - 1
    // But we need to update fast_ema from index opt_fast_period to opt_slow_period - 1
    for ad_val in ad_values.iter().take(opt_slow_period).skip(opt_fast_period) {
        fast_ema = (ad_val - fast_ema).mul_add(fast_k, fast_ema);
    }

    output_adosc[lookback] = fast_ema - slow_ema;

    // Continue for rest of the data
    for i in opt_slow_period..len {
        fast_ema = (ad_values[i] - fast_ema).mul_add(fast_k, fast_ema);
        slow_ema = (ad_values[i] - slow_ema).mul_add(slow_k, slow_ema);
        output_adosc[i] = fast_ema - slow_ema;
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
        if input_low.len() != len
            || input_close.len() != len
            || input_volume.len() != len
            || output_adosc.len() != len
        {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for adosc_val in output_adosc.iter_mut().take(len) {
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

    Ok(())
}

/// Computes the next ADOSC value incrementally using the previous EMA values.
#[inline]
pub fn adosc_inc_raw(
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    input_volume: TAFloat,
    prev_ad: TAFloat,
    prev_fast_ema: TAFloat,
    prev_slow_ema: TAFloat,
    opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    _opt_ma_type: MAType,
) -> (TAFloat, TAFloat, TAFloat, TAFloat) {
    let ad_val = ad::ad_inc_raw(input_high, input_low, input_close, input_volume, prev_ad);

    let fast_k = 2.0 / (opt_fast_period as TAFloat + 1.0);
    let slow_k = 2.0 / (opt_slow_period as TAFloat + 1.0);

    let fast_ema = (ad_val - prev_fast_ema).mul_add(fast_k, prev_fast_ema);
    let slow_ema = (ad_val - prev_slow_ema).mul_add(slow_k, prev_slow_ema);

    (fast_ema - slow_ema, ad_val, fast_ema, slow_ema)
}

/// Calculates the next ADOSC value incrementally using previous indicators' states.
///
/// # Errors
///
/// - [`KandError::InvalidParameter`] if fast or slow period is invalid.
/// - [`KandError::NaNDetected`] if any input is NaN.
pub fn adosc_inc(
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    input_volume: TAFloat,
    prev_ad: TAFloat,
    prev_fast_ema: TAFloat,
    prev_slow_ema: TAFloat,
    opt_fast_period: TAPeriod,
    opt_slow_period: TAPeriod,
    opt_ma_type: MAType,
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
            || prev_fast_ema.is_nan()
            || prev_slow_ema.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(adosc_inc_raw(
        input_high,
        input_low,
        input_close,
        input_volume,
        prev_ad,
        prev_fast_ema,
        prev_slow_ema,
        opt_fast_period,
        opt_slow_period,
        opt_ma_type,
    ))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    adosc_arrow,
    crate::ta::ohlcv::adosc::adosc_raw,
    inputs: { input_high, input_low, input_close, input_volume },
    params: { opt_fast_period: TAPeriod, opt_slow_period: TAPeriod, opt_ma_type: MAType },
    lookback_params: { opt_fast_period, opt_slow_period, opt_ma_type }
);

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use arrow::array::Array;

    use super::*;

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
            -20.897560400954944,
            -113.00659643024687,
            39.233539470168466,
            90.16827825445534,
            -42.55017122658967,
            -592.0676972749554,
            -1495.4714340938738,
            -1719.2843577555468,
            -1260.2097734422246,
            -570.3954994987735,
            -511.0214227979177,
            -1032.1203777734581,
            -1051.6428204287545,
            -697.6378360214899,
            -171.01117548929142,
            447.58320159108916,
            1478.849519342964,
            2115.784102119371,
            2377.96214942614,
            2482.3758370128967,
            2820.6011785699425,
        ];
        for (i, &expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_adosc[i + 9], expected, epsilon = 0.001);
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
            -20.897560400954944,
            -113.00659643024687,
            39.233539470168466,
            90.16827825445534,
            -42.55017122658967,
            -592.0676972749554,
            -1495.4714340938738,
            -1719.2843577555468,
            -1260.2097734422246,
            -570.3954994987735,
            -511.0214227979177,
            -1032.1203777734581,
            -1051.6428204287545,
            -697.6378360214899,
            -171.01117548929142,
            447.58320159108916,
            1478.849519342964,
            2115.784102119371,
            2377.96214942614,
            2482.3758370128967,
            2820.6011785699425,
        ];

        for (i, &expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(adosc_arrow.value(i + 9), expected, epsilon = 0.001);
        }
    }
}
