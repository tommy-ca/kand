use crate::WasmBuffer;
use kand::ta::ohlcv::rocp;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for ROCP calculation.
 */
#[wasm_bindgen(js_name = rocpLookback)]
pub fn rocp_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    rocp::lookback(opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Rate of Change Percentage (ROCP) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = rocpZeroCopy)]
pub fn rocp_wasm_zero_copy(
    input_price: &WasmBuffer,
    opt_period: usize,
    output_rocp: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    rocp::rocp(
        input_price.as_slice::<f64>(len),
        opt_period,
        output_rocp.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single ROCP value incrementally.
 */
#[wasm_bindgen(js_name = rocpInc)]
pub fn rocp_inc_wasm(input: f64, prev: f64) -> Result<f64, JsValue> {
    rocp::rocp_inc(input, prev).map_err(|e| JsValue::from_str(&e.to_string()))
}
