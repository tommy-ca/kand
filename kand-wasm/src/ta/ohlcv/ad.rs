use kand::ta::ohlcv::ad;
use crate::WasmBuffer;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for A/D calculation.
 */
#[wasm_bindgen(js_name = adLookback)]
pub fn ad_lookback_wasm() -> Result<usize, JsValue> {
    ad::lookback().map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates the Accumulation/Distribution (A/D) indicator using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = adZeroCopy)]
pub fn ad_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    input_volume: &WasmBuffer,
    output_ad: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    ad::ad(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        input_volume.as_slice::<f64>(len),
        output_ad.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates the next A/D value incrementally.
 */
#[wasm_bindgen(js_name = adInc)]
pub fn ad_inc_wasm(
    input_high: f64,
    input_low: f64,
    input_close: f64,
    input_volume: f64,
    prev_ad: f64,
) -> Result<f64, JsValue> {
    ad::ad_inc(input_high, input_low, input_close, input_volume, prev_ad)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
