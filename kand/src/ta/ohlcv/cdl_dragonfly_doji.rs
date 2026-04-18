use crate::{
    KandError, TAFloat, TAInt,
    helper::{real_body_length, upper_shadow_length},
};

/// Returns the lookback period required for Dragonfly Doji pattern detection.
///
/// # Description
/// For Dragonfly Doji patterns, no lookback is required as each candle is evaluated independently.
///
/// # Returns
/// * `Result<usize, KandError>` - Always returns `Ok(0)`
///
/// # Errors
/// This function does not return any errors.
///
/// # Example
/// ```
/// use kand::ohlcv::cdl_dragonfly_doji;
/// let lookback = cdl_dragonfly_doji::lookback().unwrap();
/// assert_eq!(lookback, 0);
/// ```
pub const fn lookback() -> Result<usize, KandError> {
    Ok(0)
}

/// Calculates Dragonfly Doji pattern without input validation for high performance.
pub fn cdl_dragonfly_doji_raw(
    input_open: &[TAFloat],
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_body_percent: TAFloat,
    opt_shadow_percent: TAFloat,
    output_signals: &mut [TAInt],
) {
    let len = input_open.len();
    for i in 0..len {
        output_signals[i] = cdl_dragonfly_doji_inc_raw(
            input_open[i],
            input_high[i],
            input_low[i],
            input_close[i],
            opt_body_percent,
            opt_shadow_percent,
        );
    }
}

/// Detects Dragonfly Doji candlestick patterns in price data.
///
/// # Description
/// A Dragonfly Doji is a bullish reversal pattern that occurs at the bottom of downtrends.
/// It is characterized by opening and closing prices near the high of the period,
/// with a long lower shadow and little to no upper shadow.
///
/// # Mathematical Formula
/// ```text
/// Body = |Close - Open|
/// Range = High - Low
/// UpperShadow = High - max(Open, Close)
/// LowerShadow = min(Open, Close) - Low
///
/// IsDragonfly = (Body <= Range * BodyPercent/100) AND
///               (UpperShadow <= Range * ShadowPercent/100) AND
///               (LowerShadow > Range * (100 - BodyPercent - ShadowPercent)/100)
/// ```
///
/// # Arguments
/// * `input_open` - Array of opening prices
/// * `input_high` - Array of high prices
/// * `input_low` - Array of low prices
/// * `input_close` - Array of closing prices
/// * `opt_body_percent` - Maximum body size as percentage of range (e.g. 5.0)
/// * `opt_shadow_percent` - Maximum upper shadow size as percentage of range (e.g. 10.0)
/// * `output_signals` - Output array for pattern signals (100 for Bullish, 0 for Neutral)
///
/// # Returns
/// * `Result<(), KandError>` - Empty result on success, or error on failure
///
/// # Errors
/// * `KandError::LengthMismatch` - If input arrays have different lengths
/// * `KandError::InvalidParameter` - If any parameter is invalid (e.g. <= 0)
/// * `KandError::NaNDetected` - If any input contains NaN values (when `check-nan` enabled)
///
/// # Example
/// ```
/// use kand::ta::ohlcv::cdl_dragonfly_doji;
///
/// let input_open = vec![10.0, 10.5, 11.0];
/// let input_high = vec![10.1, 10.6, 11.1];
/// let input_low = vec![8.5, 9.0, 10.5];
/// let input_close = vec![10.05, 10.55, 11.05];
/// let mut output_signals = vec![0; 3];
///
/// cdl_dragonfly_doji::cdl_dragonfly_doji(
///     &input_open,
///     &input_high,
///     &input_low,
///     &input_close,
///     5.0,  // opt_body_percent
///     10.0, // opt_shadow_percent
///     &mut output_signals,
/// )
/// .unwrap();
/// ```
pub fn cdl_dragonfly_doji(
    input_open: &[TAFloat],
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_body_percent: TAFloat,
    opt_shadow_percent: TAFloat,
    output_signals: &mut [TAInt],
) -> Result<(), KandError> {
    let len = input_open.len();

    #[cfg(feature = "check")]
    {
        // Check array lengths
        if len != input_high.len()
            || len != input_low.len()
            || len != input_close.len()
            || len != output_signals.len()
        {
            return Err(KandError::LengthMismatch);
        }

        // Check parameters
        if opt_body_percent <= 0.0 || opt_shadow_percent <= 0.0 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for i in 0..len {
            if input_open[i].is_nan()
                || input_high[i].is_nan()
                || input_low[i].is_nan()
                || input_close[i].is_nan()
            {
                return Err(KandError::NaNDetected);
            }
        }
    }

    cdl_dragonfly_doji_raw(
        input_open,
        input_high,
        input_low,
        input_close,
        opt_body_percent,
        opt_shadow_percent,
        output_signals,
    );

    Ok(())
}

/// Processes a single candlestick for Dragonfly Doji detection without validation.
#[inline]
pub fn cdl_dragonfly_doji_inc_raw(
    input_open: TAFloat,
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    opt_body_percent: TAFloat,
    opt_shadow_percent: TAFloat,
) -> TAInt {
    let body = real_body_length(input_open, input_close);
    let range = input_high - input_low;
    let up_shadow = upper_shadow_length(input_high, input_open, input_close);

    if range > 0.0
        && body <= range * opt_body_percent / 100.0
        && up_shadow <= range * opt_shadow_percent / 100.0
    {
        <crate::ta::types::Signal as Into<crate::TAInt>>::into(crate::ta::types::Signal::Bullish)
    } else {
        <crate::ta::types::Signal as Into<crate::TAInt>>::into(crate::ta::types::Signal::Neutral)
    }
}

/// Processes a single candlestick for Dragonfly Doji detection.
///
/// # Description
/// Optimized version for real-time analysis of individual candlesticks.
///
/// # Arguments
/// * `input_open` - Opening price
/// * `input_high` - High price
/// * `input_low` - Low price
/// * `input_close` - Closing price
/// * `opt_body_percent` - Maximum body size as percentage of range
/// * `opt_shadow_percent` - Maximum upper shadow size as percentage of range
///
/// # Returns
/// * `Result<TAInt, KandError>` - Signal value
///
/// # Errors
/// * `KandError::InvalidParameter` - If any parameter is invalid (e.g. <= 0)
/// * `KandError::NaNDetected` - If any input value is NaN
pub fn cdl_dragonfly_doji_inc(
    input_open: TAFloat,
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    opt_body_percent: TAFloat,
    opt_shadow_percent: TAFloat,
) -> Result<TAInt, KandError> {
    #[cfg(feature = "check")]
    {
        // Check parameters
        if opt_body_percent <= 0.0 || opt_shadow_percent <= 0.0 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        if input_open.is_null()
            || input_high.is_null()
            || input_low.is_null()
            || input_close.is_null()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(cdl_dragonfly_doji_inc_raw(
        input_open,
        input_high,
        input_low,
        input_close,
        opt_body_percent,
        opt_shadow_percent,
    ))
}

// Arrow wrapper
crate::kand_arrow_wrapper_int!(
    cdl_dragonfly_doji_arrow,
    crate::ta::ohlcv::cdl_dragonfly_doji::cdl_dragonfly_doji_raw,
    inputs: { input_open, input_high, input_low, input_close },
    params: { opt_body_percent: TAFloat, opt_shadow_percent: TAFloat },
    lookback_params: {}
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdl_dragonfly_doji() {
        let input_open = vec![
            98223.3, 98067.6, 97712.7, 97743.3, 97593.6, 97468.1, 96963.3, 96866.9, 97147.7,
            96845.8, 96536.0, 96337.2, 96330.0, 96440.0, 96592.4, 96662.7, 96220.0, 96111.0,
            95811.1, 96161.5, 95880.1, 96390.5, 95860.0, 95613.5, 95736.0, 96093.4, 96337.3,
            96650.8, 96609.1, 96313.0, 96050.4, 96522.0, 96036.0, 96130.0, 96313.1, 96410.4,
            96548.1, 96439.6, 96161.1, 96311.8, 96488.5, 96611.9, 96446.1, 96358.3, 96456.2,
            96600.0, 96508.0, 96700.0, 97150.0, 97021.3, 97290.0, 97333.5, 97411.4, 97355.0,
            96850.0, 96840.0, 96845.0, 96830.0, 96835.0, 97100.0, 97105.0, 97102.0,
        ];
        let input_high = vec![
            98711.4, 98454.3, 98922.1, 98356.2, 98025.4, 97952.6, 97554.0, 97285.7, 97500.0,
            97500.0, 97076.1, 96754.1, 96826.5, 96795.0, 97154.2, 96936.7, 96797.1, 96415.7,
            96430.0, 96539.7, 96530.5, 96883.1, 96412.7, 96161.9, 96327.2, 96408.3, 96781.0,
            97041.4, 96913.2, 96696.8, 96730.7, 96827.7, 96794.7, 96577.5, 96560.0, 96923.0,
            96923.0, 96638.4, 96634.5, 96576.4, 96896.7, 96896.5, 96788.3, 96563.4, 96815.0,
            96822.3, 96835.0, 97805.8, 97561.9, 97473.4, 97480.0, 97586.0, 97727.7, 97639.8,
            96850.5, 96840.5, 96845.5, 96830.5, 96835.5, 97100.5, 97105.5, 97102.5,
        ];
        let input_low = vec![
            98223.3, 98067.6, 97712.7, 97743.3, 97593.6, 97468.1, 96963.3, 96866.9, 97147.7,
            96845.8, 96536.0, 96337.2, 96330.0, 96440.0, 96592.4, 96662.7, 96220.0, 96111.0,
            95811.1, 96161.5, 95880.1, 96390.5, 95860.0, 95613.5, 95736.0, 96093.4, 96337.3,
            96650.8, 96609.1, 96313.0, 96050.4, 96522.0, 96036.0, 96130.0, 96313.1, 96410.4,
            96548.1, 96439.6, 96161.1, 96311.8, 96488.5, 96611.9, 96446.1, 96358.3, 96456.2,
            96600.0, 96508.0, 96700.0, 97150.0, 97021.3, 97290.0, 97333.5, 97411.4, 97355.0,
            96650.0, 96640.0, 96645.0, 96630.0, 96635.0, 96900.0, 96905.0, 96902.0,
        ];
        let input_close = vec![
            98563.5, 98419.9, 98249.5, 98074.7, 97797.1, 97925.6, 97546.1, 97140.3, 97285.6,
            97486.5, 97009.3, 96555.0, 96542.5, 96450.1, 96772.8, 96796.9, 96662.7, 96252.3,
            96131.1, 96364.4, 96274.1, 96447.8, 96408.3, 95960.1, 95946.1, 96238.8, 96359.0,
            96770.6, 96884.2, 96613.9, 96489.1, 96710.0, 96780.0, 96149.6, 96548.1, 96560.0,
            96923.0, 96567.2, 96571.7, 96341.5, 96515.1, 96720.2, 96746.1, 96461.0, 96460.9,
            96735.0, 96679.9, 96759.9, 97350.9, 97216.7, 97346.3, 97419.9, 97534.2, 97521.5,
            96849.5, 96839.9, 96844.7, 96829.8, 96834.6, 97099.7, 97104.8, 97101.9,
        ];

        let opt_body_percent = 5.0;
        let opt_shadow_percent = 10.0;
        let mut output_signals = vec![0; input_open.len()];

        cdl_dragonfly_doji(
            &input_open,
            &input_high,
            &input_low,
            &input_close,
            opt_body_percent,
            opt_shadow_percent,
            &mut output_signals,
        )
        .unwrap();

        // Verify specific dragonfly doji signals in the dummy extension
        let dragonfly_indices = [54, 55, 56, 57, 58, 59, 60, 61];
        for &idx in &dragonfly_indices {
            assert_eq!(
                output_signals[idx],
                <crate::ta::types::Signal as Into<crate::TAInt>>::into(
                    crate::ta::types::Signal::Bullish
                ),
                "Expected dragonfly doji signal at index {idx}"
            );
        }

        // Test incremental calculation
        for i in 0..input_open.len() {
            let signal = cdl_dragonfly_doji_inc(
                input_open[i],
                input_high[i],
                input_low[i],
                input_close[i],
                opt_body_percent,
                opt_shadow_percent,
            )
            .unwrap();
            assert_eq!(signal, output_signals[i]);
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_cdl_dragonfly_doji_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_open = vec![10.0, 10.5, 11.0];
        let input_high = vec![10.1, 10.6, 11.1];
        let input_low = vec![8.5, 9.0, 10.5];
        let input_close = vec![10.05, 10.55, 11.05];

        let open_arrow = TAArrowArray::from(input_open);
        let high_arrow = TAArrowArray::from(input_high);
        let low_arrow = TAArrowArray::from(input_low);
        let close_arrow = TAArrowArray::from(input_close);

        let result = cdl_dragonfly_doji_arrow(
            &open_arrow,
            &high_arrow,
            &low_arrow,
            &close_arrow,
            10.0,
            10.0,
        )
        .unwrap();

        assert_eq!(result.len(), 3);
    }
}
