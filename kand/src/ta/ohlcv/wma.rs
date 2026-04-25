use crate::{KandError, TAFloat};

/// Calculates the lookback period required for Weighted Moving Average (WMA).
///
/// # Description
/// The lookback period represents the minimum number of data points needed before
/// the first valid WMA value can be calculated. For WMA, this equals period - 1.
///
/// # Arguments
/// * `opt_period` - The time period for WMA calculation (must be >= 2)
///
/// # Returns
/// * `Result<usize, KandError>` - The lookback period on success
///
/// # Errors
/// * `KandError::InvalidParameter` - If period is less than 2
///
/// # Example
/// ```
/// use kand::ohlcv::wma;
///
/// let period = 14;
/// let lookback = wma::lookback(period).unwrap();
/// assert_eq!(lookback, 13); // lookback = period - 1
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

/// Calculates Weighted Moving Average (WMA) without input validation for high performance.
pub fn wma_raw(input: &[TAFloat], opt_period: usize, output: &mut [TAFloat]) {
    let len = input.len();
    let lookback = opt_period - 1;
    let denominator = (opt_period * (opt_period + 1)) as TAFloat / 2.0;

    // Calculate WMA for each window
    for i in lookback..len {
        let mut weighted_sum = 0.0;
        let mut weight = opt_period as TAFloat;

        for j in 0..opt_period {
            weighted_sum += input[i - j] * weight;
            weight -= 1.0;
        }

        output[i] = weighted_sum / denominator;
    }
}

/// Calculates Weighted Moving Average (WMA) for a price series.
///
/// # Description
/// WMA assigns linearly decreasing weights to each price in the period, giving more
/// importance to recent prices and less to older ones.
///
/// # Mathematical Formula
/// ```text
/// WMA = (P1*n + P2*(n-1) + ... + Pn*1) / (n + (n-1) + ... + 1)
/// ```
/// Where:
/// - P1, P2, ..., Pn are prices from newest to oldest
/// - n is the time period
/// - Denominator is the sum of weights: n*(n+1)/2
///
/// # Arguments
/// * `input` - Array of price values
/// * `opt_period` - The time period for WMA calculation (must be >= 2)
/// * `output` - Array to store WMA values (first period-1 values are NaN)
///
/// # Returns
/// * `Result<(), KandError>` - Empty Ok on success
///
/// # Errors
/// * `KandError::InvalidData` - If input array is empty
/// * `KandError::LengthMismatch` - If output length != input length
/// * `KandError::InvalidParameter` - If period < 2
/// * `KandError::InsufficientData` - If input length <= lookback
/// * `KandError::NaNDetected` - If any input is NaN (with `check-nan`)
///
/// # Example
/// ```
/// use kand::ohlcv::wma;
///
/// let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let mut output = vec![0.0; 5];
///
/// wma::wma(&input, 3, &mut output).unwrap();
/// // output = [NaN, NaN, 2.0, 3.0, 4.0]
/// // First value: (1.0*3 + 2.0*2 + 3.0*1)/(3+2+1) = 2.0
/// ```
pub fn wma(input: &[TAFloat], opt_period: usize, output: &mut [TAFloat]) -> Result<(), KandError> {
    let len = input.len();
    let lookback = lookback(opt_period)?;

    #[cfg(feature = "check")]
    {
        if len == 0 {
            return Err(KandError::InvalidData);
        }
        if len != output.len() {
            return Err(KandError::LengthMismatch);
        }
        if len <= lookback {
            return Err(KandError::InsufficientData);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for &value in input {
            if value.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    wma_raw(input, opt_period, output);

    // Fill initial values with NAN
    #[cfg(feature = "allow-nan")]
    {
        for value in output.iter_mut().take(lookback) {
            *value = TAFloat::NAN;
        }
    }

    Ok(())
}

/// Calculates the next WMA value incrementally without input validation.
pub fn wma_inc_raw(input_window: &[TAFloat], opt_period: usize) -> TAFloat {
    let denominator = (opt_period * (opt_period + 1)) as TAFloat / 2.0;
    let mut weighted_sum = 0.0;
    let mut weight = opt_period as TAFloat;

    for &value in input_window {
        weighted_sum += value * weight;
        weight -= 1.0;
    }

    weighted_sum / denominator
}

/// Calculates the next WMA value incrementally.
///
/// # Description
/// Computes a single WMA value for the most recent period window, useful for
/// real-time calculations without processing the entire series.
///
/// # Mathematical Formula
/// ```text
/// WMA = (P1*n + P2*(n-1) + ... + Pn*1) / (n + (n-1) + ... + 1)
/// ```
/// Where:
/// - P1, P2, ..., Pn are prices from newest to oldest
/// - n is the time period
/// - Denominator is the sum of weights: n*(n+1)/2
///
/// # Arguments
/// * `input_window` - Price values ordered from newest to oldest
/// * `opt_period` - The time period for WMA calculation (must be >= 2)
///
/// # Returns
/// * `Result<TAFloat, KandError>` - The calculated WMA value
///
/// # Errors
/// * `KandError::InvalidParameter` - If period < 2
/// * `KandError::LengthMismatch` - If `input_window` length != period
/// * `KandError::NaNDetected` - If any input is NaN (with `check-nan`)
///
/// # Example
/// ```
/// use kand::ohlcv::wma;
///
/// let window = vec![5.0, 4.0, 3.0]; // newest to oldest
/// let wma = wma::wma_inc(&window, 3).unwrap();
/// // wma = (5.0*3 + 4.0*2 + 3.0*1)/(3+2+1) = 4.333...
/// ```
pub fn wma_inc(input_window: &[TAFloat], opt_period: usize) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
        if input_window.len() != opt_period {
            return Err(KandError::LengthMismatch);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        for &value in input_window {
            if value.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    Ok(wma_inc_raw(input_window, opt_period))
}

// Arrow wrapper
#[cfg(feature = "arrow")]
crate::kand_arrow_wrapper!(
    wma_arrow,
    crate::ta::ohlcv::wma::wma_raw,
    inputs: { input },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    WMA,
    type: sliding_window,
    inputs: { price: TAFloat },
    params: { period: usize },
    state: { sum: TAFloat, weighted_sum: TAFloat },
    init: |period| {
        (0.0, 0.0)
    },
    next: |state, (price)| {
        let old_val = state.__kand_window[state.__kand_cursor].0;
        state.__kand_window[state.__kand_cursor] = (price,);
        state.__kand_cursor = (state.__kand_cursor + 1) % state.period;

        if state.__kand_count <= state.period {
            state.sum += price;
            state.weighted_sum += price * (state.__kand_count as TAFloat);
            
            if state.__kand_count == state.period {
                let denominator = (state.period * (state.period + 1)) as TAFloat / 2.0;
                Ok(state.weighted_sum / denominator)
            } else {
                Ok(TAFloat::NAN)
            }
        } else {
            state.weighted_sum = state.weighted_sum + price * (state.period as TAFloat) - state.sum;
            state.sum = state.sum + price - old_val;
            let denominator = (state.period * (state.period + 1)) as TAFloat / 2.0;
            Ok(state.weighted_sum / denominator)
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
    fn test_stateful_wma() {
        let mut wma_state = StatefulWMA::new(3).unwrap();
        assert!(wma_state.next((10.0,)).unwrap().is_nan());
        assert!(wma_state.next((10.5,)).unwrap().is_nan());
        // wma = (10.0*1 + 10.5*2 + 11.2*3) / 6 = (10.0 + 21.0 + 33.6) / 6 = 64.6 / 6 = 10.7666...
        assert_relative_eq!(wma_state.next((11.2,)).unwrap(), 10.766666666666667, epsilon = 0.0001);
        // wma = (10.5*1 + 11.2*2 + 10.8*3) / 6 = (10.5 + 22.4 + 32.4) / 6 = 65.3 / 6 = 10.88333...
        assert_relative_eq!(wma_state.next((10.8,)).unwrap(), 10.883333333333333, epsilon = 0.0001);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_wma() {
        let mut batch_wma = BatchWMA::new(3, 2).unwrap();
        let input = TAArrowArray::from(vec![10.0, 20.0]);

        // t0
        let out = batch_wma.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![10.5, 21.0]);
        let out = batch_wma.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![11.2, 22.4]);
        let out = batch_wma.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 10.766666666666667, epsilon = 0.0001);
        assert_relative_eq!(out.value(1), 21.533333333333333, epsilon = 0.0001); // (20*1 + 21*2 + 22.4*3)/6 = (20 + 42 + 67.2)/6 = 129.2/6 = 21.533...
    }

    #[test]
    fn test_wma_calculation() {
        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4,
        ];

        let opt_period = 30;
        let mut output = vec![0.0; input.len()];

        wma(&input, opt_period, &mut output).unwrap();

        // First 29 values should be NaN
        #[cfg(feature = "allow-nan")]
        for value in output.iter().take(29) {
            assert!(value.is_nan());
        }

        // Compare with known values
        let expected_values = [
            35_086.706_666_666_67,
            35_084.862_795_698_93,
            35_085.524_516_129_04,
            35_085.073_763_440_865,
            35_085.998_064_516_134,
            35_089.942_150_537_645,
            35_096.826_881_720_44,
            35_099.841_290_322_58,
            35106.98,
            35_113.904_946_236_566,
            35_117.354_193_548_395,
        ];

        for (i, expected) in expected_values.iter().enumerate() {
            assert_relative_eq!(output[i + 29], *expected, epsilon = 0.0001);
        }

        // Test incremental calculation matches regular calculation
        for i in 30..35 {
            let window: Vec<TAFloat> = input[i - (opt_period - 1)..=i]
                .iter()
                .rev()
                .copied()
                .collect();
            let result = wma_inc(&window, opt_period).unwrap();
            assert_relative_eq!(result, output[i], epsilon = 0.0001);
        }
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_wma_arrow() {
        use crate::ta::types::TAArrowArray;

        let input = vec![
            35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
            35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
            35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0, 35114.5, 35097.2,
            35092.0, 35073.2, 35139.3, 35092.0, 35126.7, 35106.3, 35124.8, 35170.1, 35215.3,
            35154.0, 35216.3, 35211.8, 35158.4,
        ];
        let input_arrow = TAArrowArray::from(input.clone());
        let opt_period = 30;

        let wma_arrow = wma_arrow(&input_arrow, opt_period).unwrap();

        assert_eq!(wma_arrow.len(), input.len());

        let mut out = vec![0.0; input.len()];
        wma(&input, opt_period, &mut out).unwrap();

        for i in 0..input.len() {
            if i < 29 {
                #[cfg(feature = "allow-nan")]
                assert!(wma_arrow.value(i).is_nan());
            } else {
                assert_relative_eq!(wma_arrow.value(i), out[i], epsilon = 0.0001);
            }
        }
    }
}
