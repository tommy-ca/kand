use crate::WasmBuffer;
use kand::ta::ohlcv::midprice;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct MidpriceResult {
    pub midprice: f64,
    pub highest_high: f64,
    pub lowest_low: f64,
}

/**
 * Calculates Midprice using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = midpriceZeroCopy)]
pub fn midprice_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    opt_period: usize,
    output_midprice: &mut WasmBuffer,
    output_highest_high: &mut WasmBuffer,
    output_lowest_low: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    midprice::midprice(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        opt_period,
        output_midprice.as_mut_slice::<f64>(len),
        output_highest_high.as_mut_slice::<f64>(len),
        output_lowest_low.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Incrementally calculates Midprice for a single period.
 */
#[wasm_bindgen(js_name = midpriceInc)]
pub fn midprice_wasm_inc(
    input_high: f64,
    input_low: f64,
    prev_highest_high: f64,
    prev_lowest_low: f64,
    opt_period: usize,
) -> Result<MidpriceResult, JsValue> {
    midprice::midprice_inc(
        input_high,
        input_low,
        prev_highest_high,
        prev_lowest_low,
        opt_period,
    )
    .map(|(midprice, highest_high, lowest_low)| MidpriceResult {
        midprice,
        highest_high,
        lowest_low,
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
