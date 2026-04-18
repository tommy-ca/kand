use crate::WasmBuffer;
use kand::ta::ohlcv::medprice;
use wasm_bindgen::prelude::*;

/**
 * Calculates Median Price (MEDPRICE) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = medpriceZeroCopy)]
pub fn medprice_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    output_buffer: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    medprice::medprice(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        output_buffer.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Incrementally calculates Median Price for a single period.
 */
#[wasm_bindgen(js_name = medpriceInc)]
pub fn medprice_wasm_inc(input_high: f64, input_low: f64) -> Result<f64, JsValue> {
    medprice::medprice_inc(input_high, input_low).map_err(|e| JsValue::from_str(&e.to_string()))
}
