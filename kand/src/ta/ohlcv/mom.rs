use crate::{KandError, TAFloat, TAPeriod};

/// Returns the lookback period for Momentum (MOM) calculation without input validation.
#[inline]
pub const fn lookback_raw(opt_period: usize) -> TAPeriod {
    opt_period
}

/// Returns the lookback period required for MOM calculation.
pub const fn lookback(opt_period: usize) -> Result<usize, KandError> {
    #[cfg(feature = "check")]
    {
        if opt_period < 1 {
            return Err(KandError::InvalidParameter);
        }
    }
    Ok(lookback_raw(opt_period))
}

/// Calculates MOM without input validation.
pub fn mom_raw(input: &[TAFloat], opt_period: usize, output: &mut [TAFloat]) {
    for i in opt_period..input.len() {
        output[i] = input[i] - input[i - opt_period];
    }
}

/// Calculates the Momentum (MOM) for a price series.
pub fn mom(input: &[TAFloat], opt_period: usize, output: &mut [TAFloat]) -> Result<(), KandError> {
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
        for price in input {
            if price.is_nan() {
                return Err(KandError::NaNDetected);
            }
        }
    }

    mom_raw(input, opt_period, output);

    // Initial values
    output[..lookback].fill(TAFloat::NAN);

    Ok(())
}

/// Calculates MOM incrementally without validation.
#[inline]
pub fn mom_inc_raw(input_current_price: TAFloat, input_old_price: TAFloat) -> TAFloat {
    input_current_price - input_old_price
}

/// Calculates MOM incrementally for a single value.
pub fn mom_inc(input_current_price: TAFloat, input_old_price: TAFloat) -> Result<TAFloat, KandError> {
    #[cfg(feature = "check-nan")]
    {
        if input_current_price.is_nan() || input_old_price.is_nan() {
            return Err(KandError::NaNDetected);
        }
    }
    Ok(mom_inc_raw(input_current_price, input_old_price))
}

// Arrow wrapper
crate::kand_arrow_wrapper!(
    mom_arrow,
    crate::ta::ohlcv::mom::mom_raw,
    inputs: { input },
    params: { opt_period: usize },
    lookback_params: { opt_period }
);

// Stateful & Batch via Universal Macro
crate::kand_indicator!(
    MOM,
    type: sliding_window,
    inputs: { input: TAFloat },
    params: { period: usize },
    state: { },
    init: |period| {
        ()
    },
    next: |state, (input)| {
        let old_val = state.__kand_window[state.__kand_cursor].0;
        state.__kand_window[state.__kand_cursor] = (input,);
        state.__kand_cursor = (state.__kand_cursor + 1) % state.period;

        if state.__kand_count <= state.period {
            Ok(TAFloat::NAN)
        } else {
            Ok(input - old_val)
        }
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ta::traits::{BatchIndicator, Indicator};
    use crate::ta::types::TAArrowArray;
    use approx::assert_relative_eq;
    use arrow::array::Array;

    #[test]
    fn test_mom_calculation() {
        let prices = vec![1.0, 2.0, 5.0, 4.0, 8.0];
        let period = 2;
        let mut output = vec![0.0; 5];

        mom(&prices, period, &mut output).unwrap();

        assert!(output[0].is_nan());
        assert!(output[1].is_nan());
        assert_relative_eq!(output[2], 4.0, epsilon = 0.0001);
        assert_relative_eq!(output[3], 2.0, epsilon = 0.0001);
        assert_relative_eq!(output[4], 3.0, epsilon = 0.0001);
    }

    #[test]
    fn test_stateful_mom() {
        let mut mom_state = StatefulMOM::new(2).unwrap();
        assert!(mom_state.next((1.0,)).unwrap().is_nan());
        assert!(mom_state.next((2.0,)).unwrap().is_nan());
        assert_relative_eq!(mom_state.next((5.0,)).unwrap(), 4.0);
        assert_relative_eq!(mom_state.next((4.0,)).unwrap(), 2.0);
    }

    #[test]
    #[cfg(feature = "arrow")]
    fn test_batch_mom() {
        let mut batch_mom = BatchMOM::new(2, 2).unwrap();
        let input = TAArrowArray::from(vec![1.0, 10.0]);

        // t0
        let out = batch_mom.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t1
        let input = TAArrowArray::from(vec![2.0, 12.0]);
        let out = batch_mom.next_batch((input.clone(),)).unwrap();
        assert!(out.value(0).is_nan());

        // t2
        let input = TAArrowArray::from(vec![5.0, 15.0]);
        let out = batch_mom.next_batch((input.clone(),)).unwrap();
        assert_relative_eq!(out.value(0), 4.0);
        assert_relative_eq!(out.value(1), 5.0);
    }
}
