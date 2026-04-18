use crate::WasmBuffer;
use kand::ta::ohlcv::roc;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for ROC calculation.
 */
#[wasm_bindgen(js_name = rocLookback)]
pub fn roc_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    roc::lookback(opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Rate of Change (ROC) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = rocZeroCopy)]
pub fn roc_wasm_zero_copy(
    input_price: &WasmBuffer,
    opt_period: usize,
    output_roc: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    roc::roc(
        input_price.as_slice::<f64>(len),
        opt_period,
        output_roc.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single ROC value incrementally.
 */
#[wasm_bindgen(js_name = rocInc)]
pub fn roc_inc_wasm(current_price: f64, prev_price: f64) -> Result<f64, JsValue> {
    roc::roc_inc(current_price, prev_price).map_err(|e| JsValue::from_str(&e.to_string()))
}
