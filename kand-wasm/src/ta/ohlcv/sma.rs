use kand::ta::ohlcv::sma;
use crate::WasmBuffer;
use wasm_bindgen::prelude::*;

/**
 * Calculates SMA using zero-copy WasmBuffers.
 * JS should write data to input_buffer, call this, then read from output_buffer.
 */
#[wasm_bindgen(js_name = smaZeroCopy)]
pub fn sma_wasm_zero_copy(
    input_buffer: &WasmBuffer,
    opt_period: usize,
    output_buffer: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_buffer.len();
    sma::sma(
        input_buffer.as_slice::<f64>(len),
        opt_period,
        output_buffer.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
