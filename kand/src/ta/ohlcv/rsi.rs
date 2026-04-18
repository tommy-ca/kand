use crate::ta::traits::Indicator;
use crate::{KandError, TAFloat};

/// Calculates the lookback period required for RSI (Relative Strength Index) calculation.
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period)
}

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    RSI,
    type: recursive,
    inputs: { input: TAFloat },
    params: { period: usize },
    state: { avg_gain: TAFloat, avg_loss: TAFloat, prev_price: TAFloat },
    init: |period| {
        (0.0, 0.0, 0.0)
    },
    next: |state, (input)| {
        if state.__kand_count == 1 {
            state.prev_price = input;
            Ok(TAFloat::NAN)
        } else if state.__kand_count <= state.period {
            let diff = input - state.prev_price;
            if diff > 0.0 {
                state.avg_gain += diff;
            } else {
                state.avg_loss += diff.abs();
            }
            state.prev_price = input;

            if state.__kand_count == state.period + 1 {
                state.avg_gain /= state.period as TAFloat;
                state.avg_loss /= state.period as TAFloat;

                if state.avg_loss == 0.0 {
                    Ok(100.0)
                } else {
                    let rs = state.avg_gain / state.avg_loss;
                    Ok(100.0 - (100.0 / (1.0 + rs)))
                }
            } else {
                Ok(TAFloat::NAN)
            }
        } else {
            let diff = input - state.prev_price;
            let (curr_gain, curr_loss) = if diff > 0.0 {
                (diff, 0.0)
            } else {
                (0.0, diff.abs())
            };

            let smoothing = state.period as TAFloat;
            state.avg_gain = state.avg_gain.mul_add(smoothing - 1.0, curr_gain) / smoothing;
            state.avg_loss = state.avg_loss.mul_add(smoothing - 1.0, curr_loss) / smoothing;
            state.prev_price = input;

            if state.avg_loss == 0.0 {
                Ok(100.0)
            } else {
                let rs = state.avg_gain / state.avg_loss;
                Ok(100.0 - (100.0 / (1.0 + rs)))
            }
        }
    }
);

/// Calculates RSI without input validation for high performance.
pub fn rsi_raw(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_rsi: &mut [TAFloat],
    output_avg_gain: &mut [TAFloat],
    output_avg_loss: &mut [TAFloat],
) {
    let len = input_prices.len();
    let lookback = opt_period;

    let mut gains = 0.0;
    let mut losses = 0.0;

    // Calculate initial gains/losses sum
    for i in 1..=lookback {
        let diff = input_prices[i] - input_prices[i - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses += diff.abs();
        }
    }

    // Calculate first RSI value
    let first_avg_gain = gains / opt_period as TAFloat;
    let first_avg_loss = losses / opt_period as TAFloat;

    output_avg_gain[lookback] = first_avg_gain;
    output_avg_loss[lookback] = first_avg_loss;

    if first_avg_loss == 0.0 {
        output_rsi[lookback] = 100.0;
    } else {
        let rs = first_avg_gain / first_avg_loss;
        output_rsi[lookback] = 100.0 - (100.0 / (1.0 + rs));
    }

    // Calculate remaining RSI values using smoothed averages
    let mut prev_avg_gain = first_avg_gain;
    let mut prev_avg_loss = first_avg_loss;
    let smoothing = opt_period as TAFloat;

    for i in lookback + 1..len {
        let diff = input_prices[i] - input_prices[i - 1];
        let (curr_gain, curr_loss) = if diff > 0.0 {
            (diff, 0.0)
        } else {
            (0.0, diff.abs())
        };

        let curr_avg_gain = prev_avg_gain.mul_add(smoothing - 1.0, curr_gain) / smoothing;
        let curr_avg_loss = prev_avg_loss.mul_add(smoothing - 1.0, curr_loss) / smoothing;

        output_avg_gain[i] = curr_avg_gain;
        output_avg_loss[i] = curr_avg_loss;

        if curr_avg_loss == 0.0 {
            output_rsi[i] = 100.0;
        } else {
            let rs = curr_avg_gain / curr_avg_loss;
            output_rsi[i] = 100.0 - (100.0 / (1.0 + rs));
        }

        prev_avg_gain = curr_avg_gain;
        prev_avg_loss = curr_avg_loss;
    }
}

/// Calculates Relative Strength Index (RSI) for a price series.
pub fn rsi(
    input_prices: &[TAFloat],
    opt_period: usize,
    output_rsi: &mut [TAFloat],
    output_avg_gain: &mut [TAFloat],
    output_avg_loss: &mut [TAFloat],
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
        if output_rsi.len() != len || output_avg_gain.len() != len || output_avg_loss.len() != len {
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

    rsi_raw(
        input_prices,
        opt_period,
        output_rsi,
        output_avg_gain,
        output_avg_loss,
    );

    // Fill initial values with NAN
    output_rsi[..lookback].fill(TAFloat::NAN);
    output_avg_gain[..lookback].fill(TAFloat::NAN);
    output_avg_loss[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Calculates the latest RSI value incrementally without validation.
pub fn rsi_inc_raw(
    input_curr_price: TAFloat,
    prev_price: TAFloat,
    prev_avg_gain: TAFloat,
    prev_avg_loss: TAFloat,
    opt_period: usize,
) -> (TAFloat, TAFloat, TAFloat) {
    let diff = input_curr_price - prev_price;
    let (curr_gain, curr_loss) = if diff > 0.0 {
        (diff, 0.0)
    } else {
        (0.0, diff.abs())
    };

    let smoothing = opt_period as TAFloat;
    let output_avg_gain = prev_avg_gain.mul_add(smoothing - 1.0, curr_gain) / smoothing;
    let output_avg_loss = prev_avg_loss.mul_add(smoothing - 1.0, curr_loss) / smoothing;

    let output_rsi = if output_avg_loss == 0.0 {
        100.0
    } else {
        let rs = output_avg_gain / output_avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    };

    (output_rsi, output_avg_gain, output_avg_loss)
}

/// Calculates the latest RSI value incrementally using previous average gain and loss values.
pub fn rsi_inc(
    input_curr_price: TAFloat,
    prev_price: TAFloat,
    prev_avg_gain: TAFloat,
    prev_avg_loss: TAFloat,
    opt_period: usize,
) -> Result<(TAFloat, TAFloat, TAFloat), KandError> {
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
        if input_curr_price.is_nan()
            || prev_price.is_nan()
            || prev_avg_gain.is_nan()
            || prev_avg_loss.is_nan()
        {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(rsi_inc_raw(
        input_curr_price,
        prev_price,
        prev_avg_gain,
        prev_avg_loss,
        opt_period,
    ))
}

#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper_multi!(
    rsi_arrow,
    crate::ta::ohlcv::rsi::rsi_raw,
    inputs: { input_prices },
    params: { opt_period: usize },
    lookback_params: { opt_period },
    outputs: { output_rsi: crate::TAFloat, output_avg_gain: crate::TAFloat, output_avg_loss: crate::TAFloat },
    return_type: { crate::ta::types::TAArrowArray, crate::ta::types::TAArrowArray, crate::ta::types::TAArrowArray }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    // Basic functionality tests
    #[test]
    fn test_stateful_rsi() {
        let mut rsi_state = StatefulRSI::new(3).unwrap();

        // Need 1 (price) + 3 (lookback) = 4 values for first valid
        assert!(rsi_state.next((10.0,)).unwrap().is_nan());
        assert!(rsi_state.next((11.0,)).unwrap().is_nan());
        assert!(rsi_state.next((12.0,)).unwrap().is_nan());
        let val = rsi_state.next((13.0,)).unwrap();
        // Gains: 1, 1, 1 -> avg = 1.0
        // Losses: 0, 0, 0 -> avg = 0.0
        assert_relative_eq!(val, 100.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_rsi() {
        let mut batch_rsi = BatchRSI::new(3, 2).unwrap();

        // t0
        let input = TAArrowArray::from(vec![10.0, 20.0]);
        let out = batch_rsi.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_rsi.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_rsi.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t3 - first valid
        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let out = batch_rsi.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 100.0);
        assert_relative_eq!(out.value(1), 100.0);
    }

    #[test]
    fn test_rsi_calculation() {
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0,
        ];
        let opt_period = 14;
        let mut output_rsi = vec![0.0; input_prices.len()];
        let mut output_avg_gain = vec![0.0; input_prices.len()];
        let mut output_avg_loss = vec![0.0; input_prices.len()];

        rsi(
            &input_prices,
            opt_period,
            &mut output_rsi,
            &mut output_avg_gain,
            &mut output_avg_loss,
        )
        .unwrap();

        // Verify first 14 values are NaN
        for value in output_rsi.iter().take(opt_period) {
            assert!(value.is_nan());
        }

        // Verify against known values
        assert_relative_eq!(output_rsi[14], 37.748_344_370_861_39, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[15], 34.223_538_361_225_86, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[16], 31.518_806_080_459_882, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[17], 33.425_568_632_418_2, epsilon = 0.00001);
        assert_relative_eq!(output_rsi[18], 40.465_006_259_629_995, epsilon = 0.00001);

        // Now test incremental calculation matches regular calculation
        let mut prev_avg_gain = output_avg_gain[opt_period];
        let mut prev_avg_loss = output_avg_loss[opt_period];
        let mut prev_price = input_prices[opt_period];

        // Test each incremental step
        for i in opt_period + 1..input_prices.len() {
            let (result, new_avg_gain, new_avg_loss) = rsi_inc(
                input_prices[i],
                prev_price,
                prev_avg_gain,
                prev_avg_loss,
                opt_period,
            )
            .unwrap();

            assert_relative_eq!(result, output_rsi[i], epsilon = 0.00001);
            assert_relative_eq!(new_avg_gain, output_avg_gain[i], epsilon = 0.00001);
            assert_relative_eq!(new_avg_loss, output_avg_loss[i], epsilon = 0.00001);

            prev_avg_gain = new_avg_gain;
            prev_avg_loss = new_avg_loss;
            prev_price = input_prices[i];
        }
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_rsi_arrow() {
        use crate::ta::types::TAArrowArray;
        let input_prices = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0,
        ];
        let input_prices_arrow = TAArrowArray::from(input_prices);
        let opt_period = 14;

        let (output_rsi_arrow, _output_avg_gain_arrow, _output_avg_loss_arrow) =
            rsi_arrow(&input_prices_arrow, opt_period).unwrap();

        // Verify against known values
        assert_relative_eq!(
            output_rsi_arrow.value(14),
            37.748_344_370_861_39,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            output_rsi_arrow.value(15),
            34.223_538_361_225_86,
            epsilon = 0.00001
        );
    }
}
