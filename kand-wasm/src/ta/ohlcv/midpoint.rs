use crate::WasmBuffer;
use kand::ta::ohlcv::midpoint;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct MidpointResult {
    pub midpoint: f64,
    pub highest: f64,
    pub lowest: f64,
}

/**
 * Calculates Midpoint using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = midpointZeroCopy)]
pub fn midpoint_wasm_zero_copy(
    input_price: &WasmBuffer,
    opt_period: usize,
    output_midpoint: &mut WasmBuffer,
    output_highest: &mut WasmBuffer,
    output_lowest: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    midpoint::midpoint(
        input_price.as_slice::<f64>(len),
        opt_period,
        output_midpoint.as_mut_slice::<f64>(len),
        output_highest.as_mut_slice::<f64>(len),
        output_lowest.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Incrementally calculates Midpoint for a single period.
 */
#[wasm_bindgen(js_name = midpointInc)]
pub fn midpoint_wasm_inc(
    input_price: f64,
    prev_highest: f64,
    prev_lowest: f64,
    opt_period: usize,
) -> Result<MidpointResult, JsValue> {
    midpoint::midpoint_inc(input_price, prev_highest, prev_lowest, opt_period)
        .map(|(midpoint, highest, lowest)| MidpointResult {
            midpoint,
            highest,
            lowest,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
