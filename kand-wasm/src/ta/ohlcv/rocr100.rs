use crate::WasmBuffer;
use kand::ta::ohlcv::rocr100;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for ROCR100 calculation.
 */
#[wasm_bindgen(js_name = rocr100Lookback)]
pub fn rocr100_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    rocr100::lookback(opt_period)
        .map(|v| v as usize)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Rate of Change Ratio * 100 (ROCR100) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = rocr100ZeroCopy)]
pub fn rocr100_wasm_zero_copy(
    input_price: &WasmBuffer,
    opt_period: usize,
    output_rocr100: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    rocr100::rocr100(
        input_price.as_slice::<f64>(len),
        opt_period,
        output_rocr100.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single ROCR100 value incrementally.
 */
#[wasm_bindgen(js_name = rocr100Inc)]
pub fn rocr100_inc_wasm(
    input: f64,
    prev: f64,
) -> Result<f64, JsValue> {
    rocr100::rocr100_inc(input, prev)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
