use crate::WasmBuffer;
use kand::ta::ohlcv::rocr;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for ROCR calculation.
 */
#[wasm_bindgen(js_name = rocrLookback)]
pub fn rocr_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    rocr::lookback(opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Rate of Change Ratio (ROCR) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = rocrZeroCopy)]
pub fn rocr_wasm_zero_copy(
    input_price: &WasmBuffer,
    opt_period: usize,
    output_rocr: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    rocr::rocr(
        input_price.as_slice::<f64>(len),
        opt_period,
        output_rocr.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single ROCR value incrementally.
 */
#[wasm_bindgen(js_name = rocrInc)]
pub fn rocr_inc_wasm(input: f64, prev: f64) -> Result<f64, JsValue> {
    rocr::rocr_inc(input, prev).map_err(|e| JsValue::from_str(&e.to_string()))
}
