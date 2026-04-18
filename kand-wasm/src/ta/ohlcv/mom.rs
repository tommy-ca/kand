use crate::WasmBuffer;
use kand::ta::ohlcv::mom;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for MOM calculation.
 */
#[wasm_bindgen(js_name = momLookback)]
pub fn mom_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    mom::lookback(opt_period)
        .map(|v| v as usize)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Momentum (MOM) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = momZeroCopy)]
pub fn mom_wasm_zero_copy(
    input_prices: &WasmBuffer,
    opt_period: usize,
    output_mom: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_prices.len();
    mom::mom(
        input_prices.as_slice::<f64>(len),
        opt_period,
        output_mom.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single Momentum (MOM) value incrementally.
 */
#[wasm_bindgen(js_name = momInc)]
pub fn mom_inc_wasm(
    input_current_price: f64,
    input_old_price: f64,
) -> Result<f64, JsValue> {
    mom::mom_inc(input_current_price, input_old_price)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
