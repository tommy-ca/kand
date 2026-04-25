use super::{sma, typprice};
use crate::{KandError, TAFloat};

/// Returns the lookback period required for CCI calculation.
///
/// # Description
/// The lookback period represents the minimum number of data points needed before
/// the first valid output can be calculated. For CCI, this equals the specified
/// period parameter minus 1.
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
/// use kand::ta::ohlcv::cci;
/// let period = 14;
/// let lookback = cci::lookback(period).unwrap();
/// assert_eq!(lookback, 13);
/// ```
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period - 1)
}

/// Calculates CCI without input validation for high performance.
pub fn cci_raw(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_period: usize,
    output_cci: &mut [TAFloat],
) {
    let len = input_high.len();
    let lookback = opt_period - 1;

    let mut output_tp = vec![0.0; len];
    let mut output_tp_sma = vec![0.0; len];

    // Calculate typical prices
    typprice::typprice_raw(input_high, input_low, input_close, &mut output_tp);

    // Calculate SMA of typical prices
    sma::sma_raw(&output_tp, opt_period, &mut output_tp_sma);

    // Calculate mean deviation
    let factor = 0.015;
    for i in lookback..len {
        let mut mean_dev = 0.0;
        for j in 0..opt_period {
            mean_dev += (output_tp[i - j] - output_tp_sma[i]).abs();
        }
        mean_dev /= opt_period as TAFloat;

        // Calculate CCI
        output_cci[i] = if mean_dev == 0.0 {
            0.0
        } else {
            (output_tp[i] - output_tp_sma[i]) / (factor * mean_dev)
        };
    }
}

/// Calculates the Commodity Channel Index (CCI) for a price series.
///
/// # Description
/// The CCI is a momentum-based oscillator that helps identify overbought and oversold
/// conditions by measuring the deviation of an asset's price from its statistical mean.
///
/// # Mathematical Formula
/// ```text
/// Typical Price (TP) = (High + Low + Close) / 3
/// Mean Deviation = Σ|TP - SMA(TP)| / n
/// CCI = (TP - SMA(TP)) / (0.015 * Mean Deviation)
/// ```
/// where:
/// - n is the period
/// - SMA(TP) is the Simple Moving Average of Typical Price over n periods
/// - 0.015 is a constant scaling factor
///
/// # Calculation Steps
/// 1. Calculate Typical Price for each period
/// 2. Calculate SMA of Typical Prices
/// 3. Calculate Mean Deviation
/// 4. Apply CCI formula using constant factor 0.015
///
/// # Arguments
/// * `input_high` - High prices array
/// * `input_low` - Low prices array
/// * `input_close` - Close prices array
/// * `opt_period` - The time period for calculations (must be >= 2)
/// * `output_cci` - Buffer to store CCI values
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
/// use kand::ohlcv::cci;
///
/// let input_high = vec![24.20, 24.07, 24.04, 23.87, 23.67];
/// let input_low = vec![23.85, 23.72, 23.64, 23.37, 23.46];
/// let input_close = vec![23.89, 23.95, 23.67, 23.78, 23.50];
/// let period = 3;
/// let mut output_cci = vec![0.0; 5];
///
/// cci::cci(
///     &input_high,
///     &input_low,
///     &input_close,
///     period,
///     &mut output_cci,
/// )
/// .unwrap();
/// ```
pub fn cci(
    input_high: &[TAFloat],
    input_low: &[TAFloat],
    input_close: &[TAFloat],
    opt_period: usize,
    output_cci: &mut [TAFloat],
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
        if len != input_low.len() || len != input_close.len() || len != output_cci.len() {
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

    cci_raw(input_high, input_low, input_close, opt_period, output_cci);

    // Fill output array with NAN initially
    #[cfg(feature = "allow-nan")]
    {
        for i in 0..lookback {
            output_cci[i] = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Calculates the next CCI value incrementally without validation.
pub fn cci_inc_raw(
    prev_sma_tp: TAFloat,
    input_new_high: TAFloat,
    input_new_low: TAFloat,
    input_new_close: TAFloat,
    input_old_high: TAFloat,
    input_old_low: TAFloat,
    input_old_close: TAFloat,
    opt_period: usize,
    tp_buffer: &mut Vec<TAFloat>,
) -> TAFloat {
    let new_tp = (input_new_high + input_new_low + input_new_close) / 3.0;
    let old_tp = (input_old_high + input_old_low + input_old_close) / 3.0;

    let sma_tp = sma::sma_inc_raw(new_tp, old_tp, prev_sma_tp, opt_period);

    if tp_buffer.len() == opt_period {
        tp_buffer.remove(0);
    }
    tp_buffer.push(new_tp);

    let mut mean_dev = 0.0;
    for &tp in tp_buffer.iter() {
        mean_dev += (tp - sma_tp).abs();
    }
    mean_dev /= opt_period as TAFloat;

    let factor = 0.015;
    if mean_dev.abs() <= TAFloat::EPSILON {
        0.0
    } else {
        (new_tp - sma_tp) / (factor * mean_dev)
    }
}

/// Calculates the next CCI value using an incremental approach.
///
/// # Description
/// This function provides an optimized way to calculate the next CCI value when new data
/// arrives, without recalculating the entire series. It maintains a circular buffer of
/// typical prices to ensure exact match with batch calculation.
///
/// # Mathematical Formula
/// ```text
/// Typical Price (TP) = (High + Low + Close) / 3
/// Next SMA(TP) = Previous SMA(TP) + (New TP - Old TP) / n
/// Mean Deviation = Σ|TP - SMA(TP)| / n
/// CCI = (TP - SMA(TP)) / (0.015 * Mean Deviation)
/// ```
///
/// # Calculation Steps
/// 1. Calculate new and old typical prices
/// 2. Update SMA using incremental formula
/// 3. Update circular buffer and recalculate mean deviation
/// 4. Apply CCI formula with constant factor 0.015
///
/// # Arguments
/// * `prev_sma_tp` - Previous SMA value of typical prices
/// * `input_new_high` - New high price
/// * `input_new_low` - New low price
/// * `input_new_close` - New close price
/// * `input_old_high` - Old high price to be removed
/// * `input_old_low` - Old low price to be removed
/// * `input_old_close` - Old close price to be removed
/// * `opt_period` - The time period for calculations (must be >= 2)
/// * `tp_buffer` - Circular buffer containing last `opt_period` typical prices
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The next CCI value on success, or error on failure
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
/// * `KandError::NaNDetected` - If any input contains NaN values
///
/// # Example
/// ```
/// use kand::ta::ohlcv::cci;
///
/// let prev_sma_tp = 100.0;
/// let new_high = 105.0;
/// let new_low = 95.0;
/// let new_close = 100.0;
/// let old_high = 102.0;
/// let old_low = 98.0;
/// let old_close = 100.0;
/// let period = 14;
/// let mut tp_buffer = vec![100.0; period];
///
/// let next_cci = cci::cci_inc(
///     prev_sma_tp,
///     new_high,
///     new_low,
///     new_close,
///     old_high,
///     old_low,
///     old_close,
///     period,
///     &mut tp_buffer,
/// )
/// .unwrap();
/// ```
pub fn cci_inc(
    prev_sma_tp: TAFloat,
    input_new_high: TAFloat,
    input_new_low: TAFloat,
    input_new_close: TAFloat,
    input_old_high: TAFloat,
    input_old_low: TAFloat,
    input_old_close: TAFloat,
    opt_period: usize,
    tp_buffer: &mut Vec<TAFloat>,
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
        if prev_sma_tp.is_null()
            || input_new_high.is_null()
            || input_new_low.is_null()
            || input_new_close.is_null()
            || input_old_high.is_null()
            || input_old_low.is_null()
            || input_old_close.is_null()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(cci_inc_raw(
        prev_sma_tp,
        input_new_high,
        input_new_low,
        input_new_close,
        input_old_high,
        input_old_low,
        input_old_close,
        opt_period,
        tp_buffer,
    ))
}

// Arrow wrapper
#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    cci_arrow,
    crate::ta::ohlcv::cci::cci_raw,
    inputs: { input_high, input_low, input_close },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    CCI,
    type: sliding_window,
    inputs: { high: TAFloat, low: TAFloat, close: TAFloat },
    params: { period: usize },
    state: { sum: TAFloat },
    init: |period| {
        (0.0)
    },
    next: |state, (high, low, close)| {
        let tp = (high + low + close) / 3.0;
        let (old_high, old_low, old_close) = state.__kand_window[state.__kand_cursor];
        let old_tp = (old_high + old_low + old_close) / 3.0;
        state.__kand_window[state.__kand_cursor] = (high, low, close);
        state.__kand_cursor = (state.__kand_cursor + 1) % state.period;

        if state.__kand_count <= state.period {
            state.sum += tp;
            if state.__kand_count == state.period {
                let sma_tp = state.sum / state.period as TAFloat;
                let mut mean_dev = 0.0;
                for i in 0..state.period {
                    let (h, l, c) = state.__kand_window[i];
                    mean_dev += ((h + l + c) / 3.0 - sma_tp).abs();
                }
                mean_dev /= state.period as TAFloat;
                if mean_dev == 0.0 {
                    Ok(0.0)
                } else {
                    Ok((tp - sma_tp) / (0.015 * mean_dev))
                }
            } else {
                Ok(TAFloat::NAN)
            }
        } else {
            state.sum = state.sum + tp - old_tp;
            let sma_tp = state.sum / state.period as TAFloat;
            let mut mean_dev = 0.0;
            for i in 0..state.period {
                let (h, l, c) = state.__kand_window[i];
                mean_dev += ((h + l + c) / 3.0 - sma_tp).abs();
            }
            mean_dev /= state.period as TAFloat;
            if mean_dev == 0.0 {
                Ok(0.0)
            } else {
                Ok((tp - sma_tp) / (0.015 * mean_dev))
            }
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
    fn test_stateful_cci() {
        let mut cci_state = StatefulCCI::new(3).unwrap();
        assert!(cci_state.next((24.20, 23.85, 23.89)).unwrap().is_nan());
        assert!(cci_state.next((24.07, 23.72, 23.95)).unwrap().is_nan());
        assert_relative_eq!(cci_state.next((24.04, 23.64, 23.67)).unwrap(), -100.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_cci() {
        let mut batch_cci = BatchCCI::new(3, 2).unwrap();
        let high = TAArrowArray::from(vec![24.20, 24.20]);
        let low = TAArrowArray::from(vec![23.85, 23.85]);
        let close = TAArrowArray::from(vec![23.89, 23.89]);

        // t0
        let out = batch_cci.next_batch((high.clone(), low.clone(), close.clone())).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let high = TAArrowArray::from(vec![24.07, 24.07]);
        let low = TAArrowArray::from(vec![23.72, 23.72]);
        let close = TAArrowArray::from(vec![23.95, 23.95]);
        let out = batch_cci.next_batch((high.clone(), low.clone(), close.clone())).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let high = TAArrowArray::from(vec![24.04, 24.04]);
        let low = TAArrowArray::from(vec![23.64, 23.64]);
        let close = TAArrowArray::from(vec![23.67, 23.67]);
        let out = batch_cci.next_batch((high.clone(), low.clone(), close.clone())).unwrap();
        assert_relative_eq!(out.value(0), -100.0, epsilon = 0.0001);
    }


    #[test]
    fn test_cci_calculation() {
        let input_high = vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35010.8, 35047.1, 35091.4,
        ];
        let input_low = vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35203.5, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34911.0, 34952.5, 34977.9, 35039.0,
        ];
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];
        let opt_period = 14;
        let mut output_cci = vec![0.0; input_high.len()];

        cci(
            &input_high,
            &input_low,
            &input_close,
            opt_period,
            &mut output_cci,
        )
        .unwrap();

        // First 13 values should be NaN
        #[cfg(feature = "allow-nan")]
        for i in 0..13 {
            assert!(output_cci[i].is_nan());
        }

        // Compare with known values
        let expected_values = [
            -94.082_890_723_346_37,
            -180.802_792_321_114_62,
            -244.063_557_150_198_87,
            -243.848_383_823_747_3,
            -166.790_215_765_872_72,
            -89.041_824_371_64,
            -81.225_924_313_890_73,
            -119.920_356_473_813_2,
            -114.051_248_309_390_3,
            -74.418_873_070_067_66,
            -41.113_546_460_345_28,
            7.295_737_949_004_944,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_cci[i + 13], *expected, epsilon = 0.0001);
        }

        // Initialize circular buffer for incremental calculation
        let mut tp_buffer = Vec::with_capacity(opt_period);
        for i in 0..opt_period {
            let tp = (input_high[i] + input_low[i] + input_close[i]) / 3.0;
            tp_buffer.push(tp);
        }

        // For testing incremental SMA
        let mut tp_values = vec![0.0; input_high.len()];
        typprice::typprice_raw(&input_high, &input_low, &input_close, &mut tp_values);
        let mut tp_sma = vec![0.0; input_high.len()];
        sma::sma_raw(&tp_values, opt_period, &mut tp_sma);

        // Calculate and verify incremental values
        for i in opt_period..input_high.len() {
            // Calculate incremental CCI
            let result = cci_inc(
                tp_sma[i - 1],
                input_high[i],
                input_low[i],
                input_close[i],
                input_high[i - opt_period],
                input_low[i - opt_period],
                input_close[i - opt_period],
                opt_period,
                &mut tp_buffer,
            )
            .unwrap();

            // Compare with full calculation
            assert_relative_eq!(result, output_cci[i], epsilon = 0.00001);
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_cci_arrow() {
        use crate::ta::types::TAArrowArray;

        let input_high = vec![
            35266.0, 35247.5, 35235.7, 35190.8, 35182.0, 35258.0, 35262.9, 35281.5, 35256.0,
            35210.0, 35185.4, 35230.0, 35241.0, 35218.1, 35212.6, 35128.9, 35047.7, 35019.5,
            35078.8, 35085.0, 35034.1, 34984.4, 35010.8, 35047.1, 35091.4,
        ];
        let input_low = vec![
            35216.1, 35206.5, 35180.0, 35130.7, 35153.6, 35174.7, 35202.6, 35203.5, 35175.0,
            35166.0, 35170.9, 35154.1, 35186.0, 35143.9, 35080.1, 35021.1, 34950.1, 34966.0,
            35012.3, 35022.2, 34931.6, 34911.0, 34952.5, 34977.9, 35039.0,
        ];
        let input_close = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
        ];

        let high_arrow = TAArrowArray::from(input_high.clone());
        let low_arrow = TAArrowArray::from(input_low.clone());
        let close_arrow = TAArrowArray::from(input_close.clone());
        let opt_period = 14;

        let cci_arrow = cci_arrow(&high_arrow, &low_arrow, &close_arrow, opt_period).unwrap();

        assert_eq!(cci_arrow.len(), input_high.len());

        let mut out_cci = vec![0.0; input_high.len()];

        cci(
            &input_high,
            &input_low,
            &input_close,
            opt_period,
            &mut out_cci,
        )
        .unwrap();

        for i in 0..input_high.len() {
            if i < 13 {
                #[cfg(feature = "allow-nan")]
                assert!(cci_arrow.is_null(i));
            } else {
                assert_relative_eq!(cci_arrow.value(i), out_cci[i], epsilon = 0.0001);
            }
        }
    }
}
