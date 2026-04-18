use crate::WasmBuffer;
use kand::ta::ohlcv::rma;
use wasm_bindgen::prelude::*;

/**
 * Returns the lookback period for RMA calculation.
 */
#[wasm_bindgen(js_name = rmaLookback)]
pub fn rma_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    rma::lookback(opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Running Moving Average (RMA) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = rmaZeroCopy)]
pub fn rma_wasm_zero_copy(
    input_price: &WasmBuffer,
    opt_period: usize,
    output_rma: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    rma::rma(
        input_price.as_slice::<f64>(len),
        opt_period,
        output_rma.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single RMA value incrementally.
 */
#[wasm_bindgen(js_name = rmaInc)]
pub fn rma_inc_wasm(input_current: f64, prev_rma: f64, opt_period: usize) -> Result<f64, JsValue> {
    rma::rma_inc(input_current, prev_rma, opt_period).map_err(|e| JsValue::from_str(&e.to_string()))
}
