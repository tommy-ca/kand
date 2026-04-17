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
    sma::sma(&input_buffer.data, opt_period, &mut output_buffer.data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
