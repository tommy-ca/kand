use crate::{KandError, TAFloat, TAPeriod, helper::period_to_k};

/// Returns the lookback period for EMA without input validation.
#[inline]
pub const fn lookback_raw(opt_period: TAPeriod) -> TAPeriod {
    opt_period - 1
}

/// Returns the lookback period required for EMA calculation.
pub const fn lookback(opt_period: TAPeriod) -> Result<TAPeriod, KandError> {
    #[cfg(feature = "check")]
    {
        // Parameter range check
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(lookback_raw(opt_period))
}

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    EMA,
    type: recursive,
    inputs: { price: TAFloat },
    params: { period: TAPeriod, multiplier: TAFloat },
    state: { prev_ema: TAFloat, sum: TAFloat },
    init: |period, multiplier| {
        (0.0, 0.0)
    },
    next: |state, (price)| {
        if price.is_nan() {
            state.__kand_count -= 1; // Don't count NaN
            return Ok(TAFloat::NAN);
        }

        if state.__kand_count < state.period {
            state.sum += price;
            Ok(TAFloat::NAN)
        } else if state.__kand_count == state.period {
            state.sum += price;
            state.prev_ema = state.sum / state.period as TAFloat;
            Ok(state.prev_ema)
        } else {
            state.prev_ema = (price - state.prev_ema).mul_add(state.multiplier, state.prev_ema);
            Ok(state.prev_ema)
        }
    }
);

// We need to override the macro's default 'new' for EMA to handle Option<k>
impl StatefulEMA {
    pub fn new_ext(period: TAPeriod, opt_k: Option<TAFloat>) -> Result<Self, KandError> {
        #[cfg(feature = "check")]
        {
            if period < 2 {
                return Err(KandError::InvalidParameter);
            }
        }
        let multiplier = match opt_k {
            Some(k) => k,
            None => 2.0 / (period as TAFloat + 1.0),
        };
        Self::new(period, multiplier)
    }
}

#[cfg(feature = "arrow")]
impl BatchEMA {
    pub fn new_ext(period: TAPeriod, num_streams: usize, opt_k: Option<TAFloat>) -> Result<Self, KandError> {
        #[cfg(feature = "check")]
        {
            if period < 2 || num_streams == 0 {
                return Err(KandError::InvalidParameter);
            }
        }
        let multiplier = match opt_k {
            Some(k) => k,
            None => 2.0 / (period as TAFloat + 1.0),
        };
        Self::new(period, multiplier, num_streams)
    }
}


/// Computes EMA without input validation for high performance.
pub fn ema_raw(
    input_prices: &[TAFloat],
    opt_period: TAPeriod,
    opt_k: Option<TAFloat>,
    output_ema: &mut [TAFloat],
) {
    let len = input_prices.len();
    let lookback = lookback_raw(opt_period);

    // Calculate initial SMA
    let sum: TAFloat = input_prices.iter().take(opt_period).sum();
    let mut prev_ma = sum / (opt_period as TAFloat);
    output_ema[lookback] = prev_ma;

    // Get multiplier - either custom or default
    let multiplier = match opt_k {
        Some(k) => k,
        None => 2.0 / (opt_period + 1) as TAFloat,
    };

    // Calculate EMA
    for i in opt_period..len {
        prev_ma = (input_prices[i] - prev_ma).mul_add(multiplier, prev_ma);
        output_ema[i] = prev_ma;
    }
}

/// Calculates Exponential Moving Average (EMA) for a price series.
pub fn ema(
    input_prices: &[TAFloat],
    opt_period: TAPeriod,
    opt_k: Option<TAFloat>,
    output_ema: &mut [TAFloat],
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
        if output_ema.len() != len {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for price in input_prices {
            // NaN check
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    ema_raw(input_prices, opt_period, opt_k, output_ema);

    // Fill initial values with NAN
    output_ema[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Computes the next EMA value incrementally without input validation.
#[inline]
pub fn ema_inc_raw(input_price: TAFloat, prev_ema: TAFloat, multiplier: TAFloat) -> TAFloat {
    (input_price - prev_ema).mul_add(multiplier, prev_ema)
}

/// Calculates a single EMA value incrementally using the previous EMA.
pub fn ema_inc(
    input_price: TAFloat,
    prev_ema: TAFloat,
    opt_period: TAPeriod,
    opt_k: Option<TAFloat>,
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
        if input_price.is_nan() || prev_ema.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    let multiplier = match opt_k {
        Some(k) => k,
        None => period_to_k(opt_period)?,
    };
    Ok(ema_inc_raw(input_price, prev_ema, multiplier))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    ema_arrow,
    crate::ta::ohlcv::ema::ema_raw,
    inputs: { input_prices },
    params: { opt_period: TAPeriod, opt_k: Option<TAFloat> },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use arrow::array::Array;

    use super::*;

    // Basic functionality tests
    #[test]
    fn test_stateful_ema() {
        use crate::ta::traits::Indicator;
        let mut ema = StatefulEMA::new_ext(3, None).unwrap();
        assert!(ema.next((10.0,)).unwrap().is_nan());
        assert!(ema.next((11.0,)).unwrap().is_nan());
        assert_relative_eq!(ema.next((12.0,)).unwrap(), 11.0);
        assert_relative_eq!(ema.next((13.0,)).unwrap(), 12.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_ema() {
        use crate::ta::traits::BatchIndicator;
        use crate::ta::types::TAArrowArray;
        let mut batch_ema = BatchEMA::new_ext(3, 2, None).unwrap();

        // t0
        let input = TAArrowArray::from(vec![10.0, 20.0]);
        let out = batch_ema.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t1
        let input = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_ema.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t2 - first valid (SMA)
        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_ema.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 11.0);
        assert_relative_eq!(out.value(1), 21.0);

        // t3 - EMA update
        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let out = batch_ema.next_batch((input.clone(),)).unwrap();
        // k = 2/(3+1) = 0.5
        // ema = (13 - 11) * 0.5 + 11 = 12.0
        assert_relative_eq!(out.value(0), 12.0);
        assert_relative_eq!(out.value(1), 22.0);
    }

    #[test]
    fn test_ema_calculation() {
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
        ];
        let opt_period = 14;
        let mut output_ema = vec![0.0; input_prices.len()];

        ema(&input_prices, opt_period, None, &mut output_ema).unwrap();

        // First 13 values should be NaN
        for value in output_ema.iter().take(13) {
            assert!(value.is_nan());
        }

        // Test first valid value
        let expected_values = [
            35_203.535_714_285_72,
            35_188.437_619_047_625,
            35_168.805_936_507_94,
            35_146.205_144_973_545,
            35_128.497_792_310_41,
            35_120.564_753_335_69,
            35_107.769_452_890_934,
            35_085.333_525_838_81,
            35_067.635_722_393_636,
            35_058.617_626_074_48,
            35_056.375_275_931_22,
            35_059.525_239_140_39,
            35_066.855_207_255_01,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_ema[i + 13], *expected, epsilon = 0.00001);
        }

        let opt_period = 14;
        let mut output_ema = vec![0.0; input_prices.len()];

        ema(&input_prices, opt_period, None, &mut output_ema).unwrap();

        // Now test incremental calculation matches regular calculation
        let mut prev_ema = output_ema[13]; // First valid EMA value

        // Test each incremental step
        for i in 14..18 {
            let result = ema_inc(input_prices[i], prev_ema, opt_period, None).unwrap();
            assert_relative_eq!(result, output_ema[i], epsilon = 0.00001);
            prev_ema = result;
        }
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_ema_arrow() {
        use crate::ta::types::TAArrowArray;
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
        ];
        let input_prices_arrow = TAArrowArray::from(input_prices);
        let opt_period = 14;

        let output_ema_arrow = ema_arrow(&input_prices_arrow, opt_period, None).unwrap();

        // Test first valid value
        let expected_values = [
            35_203.535_714_285_72,
            35_188.437_619_047_625,
            35_168.805_936_507_94,
            35_146.205_144_973_545,
            35_128.497_792_310_41,
            35_120.564_753_335_69,
            35_107.769_452_890_934,
            35_085.333_525_838_81,
            35_067.635_722_393_636,
            35_058.617_626_074_48,
            35_056.375_275_931_22,
            35_059.525_239_140_39,
            35_066.855_207_255_01,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output_ema_arrow.value(i + 13), *expected, epsilon = 0.00001);
        }
    }
}
