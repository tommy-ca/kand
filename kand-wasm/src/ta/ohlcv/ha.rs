use crate::WasmBuffer;
use kand::ta::ohlcv::ha;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct HAResult {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/**
 * Calculates Heikin-Ashi candlesticks using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = haZeroCopy)]
pub fn ha_wasm_zero_copy(
    input_open: &WasmBuffer,
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    output_open: &mut WasmBuffer,
    output_high: &mut WasmBuffer,
    output_low: &mut WasmBuffer,
    output_close: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_open.len();
    ha::ha(
        input_open.as_slice::<f64>(len),
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        output_open.as_mut_slice::<f64>(len),
        output_high.as_mut_slice::<f64>(len),
        output_low.as_mut_slice::<f64>(len),
        output_close.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Incrementally calculates Heikin-Ashi for a single period.
 */
#[wasm_bindgen(js_name = haInc)]
pub fn ha_wasm_inc(
    curr_open: f64,
    curr_high: f64,
    curr_low: f64,
    curr_close: f64,
    prev_ha_open: f64,
    prev_ha_close: f64,
) -> Result<HAResult, JsValue> {
    ha::ha_inc(
        curr_open,
        curr_high,
        curr_low,
        curr_close,
        prev_ha_open,
        prev_ha_close,
    )
    .map(|(open, high, low, close)| HAResult {
        open,
        high,
        low,
        close,
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
