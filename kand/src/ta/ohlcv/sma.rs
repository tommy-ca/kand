use crate::ta::traits::Indicator;
use crate::{KandError, TAFloat, TAPeriod};

/// Returns the lookback period required for Simple Moving Average (SMA).
pub const fn lookback(opt_period: TAPeriod) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(opt_period - 1)
}

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    SMA,
    type: sliding_window,
    inputs: { input: TAFloat },
    params: { period: TAPeriod },
    state: { sum: TAFloat },
    init: |period| {
        (0.0)
    },
    next: |state, (input)| {
        let old_val = state.__kand_window[state.__kand_cursor].0;
        state.__kand_window[state.__kand_cursor] = (input,);
        state.__kand_cursor = (state.__kand_cursor + 1) % state.period;

        if state.__kand_count <= state.period {
            state.sum += input;
        } else {
            state.sum += input - old_val;
        }

        if state.__kand_count < state.period {
            Ok(TAFloat::NAN)
        } else {
            Ok(state.sum / state.period as TAFloat)
        }
    }
);


/// Calculates SMA without input validation.
pub fn sma_raw(input: &[TAFloat], opt_period: TAPeriod, output: &mut [TAFloat]) {
    let mut sum: TAFloat = input.iter().take(opt_period).sum();

    output[opt_period - 1] = sum / opt_period as TAFloat;

    for i in opt_period..input.len() {
        sum += input[i] - input[i - opt_period];
        output[i] = sum / opt_period as TAFloat;
    }
}

/// Calculates the Simple Moving Average (SMA) for a given data slice.
pub fn sma(
    input: &[TAFloat],
    opt_period: TAPeriod,
    output: &mut [TAFloat],
) -> Result<(), KandError> {
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
        for i in 0..len {
            if input[i].is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    sma_raw(input, opt_period, output);

    // Initial values
    output[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Calculates SMA incrementally for a single value without validation.
#[inline]
pub fn sma_inc_raw(
    input: TAFloat,
    prev_input: TAFloat,
    prev_sma: TAFloat,
    opt_period: TAPeriod,
) -> TAFloat {
    prev_sma + (input - prev_input) / opt_period as TAFloat
}

/// Calculates SMA incrementally for a single value.
pub fn sma_inc(
    input: TAFloat,
    prev_input: TAFloat,
    prev_sma: TAFloat,
    opt_period: TAPeriod,
) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 2 {
            return Err(KandError::InvalidParameter);
        }
    }

    #[cfg(feature = "check-nan")]
    {
        if input.is_nan() || prev_input.is_nan() || prev_sma.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }

    Ok(prev_sma + (input - prev_input) / opt_period as TAFloat)
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    sma_arrow,
    crate::ta::ohlcv::sma::sma_raw,
    inputs: { input },
    params: { opt_period: TAPeriod },
    lookback_params: { opt_period }
);

#[cfg(test)]
mod tests {
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    use super::*;

    #[test]
    fn test_stateful_sma() {
        let mut sma = StatefulSMA::new(3).unwrap();
        assert!(sma.next((10.0,)).unwrap().is_nan());
        assert!(sma.next((11.0,)).unwrap().is_nan());
        assert_relative_eq!(sma.next((12.0,)).unwrap(), 11.0);
        assert_relative_eq!(sma.next((13.0,)).unwrap(), 12.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_sma() {
        let mut batch_sma = BatchSMA::new(3, 2).unwrap();
        let input = TAArrowArray::from(vec![10.0, 20.0]);

        // t0
        let out = batch_sma.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t1
        let input = TAArrowArray::from(vec![11.0, 21.0]);
        let out = batch_sma.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());
        assert!(out.value(1).is_nan());

        // t2 - first valid
        let input = TAArrowArray::from(vec![12.0, 22.0]);
        let out = batch_sma.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 11.0);
        assert_relative_eq!(out.value(1), 21.0);

        // t3
        let input = TAArrowArray::from(vec![13.0, 23.0]);
        let out = batch_sma.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 12.0);
        assert_relative_eq!(out.value(1), 22.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_sma_arrow() {
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let input_arrow = TAArrowArray::from(input);
        const PERIOD: usize = 3;

        let result = sma_arrow(&input_arrow, PERIOD).unwrap();

        assert_eq!(result.len(), 5);
        for i in 0..PERIOD - 1 {
            assert!(result.value(i).is_nan());
        }
        assert_relative_eq!(result.value(2), 2.0);
        assert_relative_eq!(result.value(3), 3.0);
        assert_relative_eq!(result.value(4), 4.0);
    }
}
