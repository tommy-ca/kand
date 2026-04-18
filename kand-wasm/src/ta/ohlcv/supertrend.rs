use crate::WasmBuffer;
use kand::TAInt;
use kand::ta::ohlcv::supertrend;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct SupertrendResult {
    pub trend: TAInt,
    pub supertrend: f64,
    pub atr: f64,
    pub upper: f64,
    pub lower: f64,
}

/**
 * Returns the lookback period for Supertrend calculation.
 */
#[wasm_bindgen(js_name = supertrendLookback)]
pub fn supertrend_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    supertrend::lookback(opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Supertrend using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = supertrendZeroCopy)]
#[allow(clippy::too_many_arguments)]
pub fn supertrend_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    opt_period: usize,
    opt_multiplier: f64,
    output_trend: &mut WasmBuffer,
    output_supertrend: &mut WasmBuffer,
    output_atr: &mut WasmBuffer,
    output_upper: &mut WasmBuffer,
    output_lower: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    supertrend::supertrend(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        opt_period,
        opt_multiplier,
        output_trend.as_mut_slice::<TAInt>(len),
        output_supertrend.as_mut_slice::<f64>(len),
        output_atr.as_mut_slice::<f64>(len),
        output_upper.as_mut_slice::<f64>(len),
        output_lower.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single Supertrend value incrementally.
 */
#[wasm_bindgen(js_name = supertrendInc)]
#[allow(clippy::too_many_arguments)]
pub fn supertrend_inc_wasm(
    input_high: f64,
    input_low: f64,
    input_close: f64,
    prev_close: f64,
    prev_atr: f64,
    prev_trend: TAInt,
    prev_upper: f64,
    prev_lower: f64,
    opt_period: usize,
    opt_multiplier: f64,
) -> Result<SupertrendResult, JsValue> {
    supertrend::supertrend_inc(
        input_high,
        input_low,
        input_close,
        prev_close,
        prev_atr,
        prev_trend,
        prev_upper,
        prev_lower,
        opt_period,
        opt_multiplier,
    )
    .map(|(trend, supertrend, atr, upper, lower)| SupertrendResult {
        trend,
        supertrend,
        atr,
        upper,
        lower,
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
