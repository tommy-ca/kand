use crate::WasmBuffer;
use kand::ta::ohlcv::wclprice;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for WCLPRICE calculation.
 */
#[wasm_bindgen(js_name = wclpriceLookback)]
pub fn wclprice_lookback_wasm() -> Result<usize, JsValue> {
    wclprice::lookback().map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Weighted Close Price (WCLPRICE) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = wclpriceZeroCopy)]
pub fn wclprice_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    output: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    wclprice::wclprice(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        output.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single WCLPRICE value incrementally.
 */
#[wasm_bindgen(js_name = wclpriceInc)]
pub fn wclprice_inc_wasm(
    input_high: f64,
    input_low: f64,
    input_close: f64,
) -> Result<f64, JsValue> {
    wclprice::wclprice_inc(input_high, input_low, input_close)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
