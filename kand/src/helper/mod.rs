use crate::{KandError, TAFloat, TAPeriod};

/// Calculates the multiplier `k` for Exponential Moving Average (EMA).
pub fn period_to_k(period: TAPeriod) -> Result<TAFloat, KandError> {
    if period == 0 {
        return Err(KandError::InvalidParameter);
    }
    Ok(2.0 / (period as TAFloat + 1.0))
}

/// Returns the real body length of a candle.
#[inline]
pub fn real_body_length(open: TAFloat, close: TAFloat) -> TAFloat {
    (open - close).abs()
}

/// Returns the upper shadow length of a candle.
#[inline]
pub fn upper_shadow_length(high: TAFloat, open: TAFloat, close: TAFloat) -> TAFloat {
    high - open.max(close)
}

/// Returns the lower shadow length of a candle.
#[inline]
pub fn lower_shadow_length(low: TAFloat, open: TAFloat, close: TAFloat) -> TAFloat {
    open.min(close) - low
}

/// Find the number of bars back to the lowest value in a lookback period
///
/// # Arguments
/// * `array` - Array of values to analyze
/// * `start_idx` - Starting index for analysis
/// * `lookback` - Number of periods to look back
///
/// # Returns
/// * `Result<usize, KandError>` - Number of bars back to the lowest value
pub fn lowest_bars(array: &[TAFloat], start_idx: usize, lookback: usize) -> Result<usize, KandError> {
    if array.is_empty() || start_idx >= array.len() || lookback == 0 {
        return Err(KandError::InvalidData);
    }

    let end = start_idx + 1;
    let start = end.saturating_sub(lookback);

    let mut min_val = array[start];
    let mut min_idx = start;

    for (i, &val) in array.iter().enumerate().take(end).skip(start + 1) {
        if val < min_val {
            min_val = val;
            min_idx = i;
        }
    }

    Ok(start_idx - min_idx)
}

/// Find the number of bars back to the highest value in a lookback period
///
/// # Arguments
/// * `array` - Array of values to analyze
/// * `start_idx` - Starting index for analysis
/// * `lookback` - Number of periods to look back
///
/// # Returns
/// * `Result<usize, KandError>` - Number of bars back to the highest value
pub fn highest_bars(
    array: &[TAFloat],
    start_idx: usize,
    lookback: usize,
) -> Result<usize, KandError> {
    if array.is_empty() || start_idx >= array.len() || lookback == 0 {
        return Err(KandError::InvalidData);
    }

    let end = start_idx + 1;
    let start = end.saturating_sub(lookback);

    let mut max_val = array[start];
    let mut max_idx = start;

    for (i, &val) in array.iter().enumerate().take(end).skip(start + 1) {
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }

    Ok(start_idx - max_idx)
}

/// Checks if there is a real body gap up between two candles
#[must_use]
pub fn has_real_body_gap_up(
    open2: TAFloat,
    close2: TAFloat,
    open1: TAFloat,
    close1: TAFloat,
) -> bool {
    open2.min(close2) > open1.max(close1)
}

/// Checks if there is a real body gap down between two candles
#[must_use]
pub fn has_real_body_gap_down(
    open2: TAFloat,
    close2: TAFloat,
    open1: TAFloat,
    close1: TAFloat,
) -> bool {
    open2.max(close2) < open1.min(close1)
}

pub mod arrow_macro;
pub mod buffer_pool;
