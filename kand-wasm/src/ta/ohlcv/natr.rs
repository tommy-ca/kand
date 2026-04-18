use crate::WasmBuffer;
use kand::ta::ohlcv::natr;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for NATR calculation.
 */
#[wasm_bindgen(js_name = natrLookback)]
pub fn natr_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    natr::lookback(opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Normalized Average True Range (NATR) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = natrZeroCopy)]
pub fn natr_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    opt_period: usize,
    output_natr: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    natr::natr(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        opt_period,
        output_natr.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single NATR value incrementally.
 */
#[wasm_bindgen(js_name = natrInc)]
pub fn natr_inc_wasm(
    input_high: f64,
    input_low: f64,
    input_close: f64,
    prev_close: f64,
    prev_atr: f64,
    opt_period: usize,
) -> Result<f64, JsValue> {
    natr::natr_inc(
        input_high,
        input_low,
        input_close,
        prev_close,
        prev_atr,
        opt_period,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
