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

/**
 * Calculates SMA using Arrow-native core and returns a new pooled WasmBuffer.
 */
#[cfg(feature = "arrow")]
#[wasm_bindgen(js_name = smaArrow)]
pub fn sma_arrow_wasm(
    input_buffer: &WasmBuffer,
    opt_period: usize,
) -> Result<WasmBuffer, JsValue> {
    use arrow::datatypes::Float64Type;

    let input_arrow = input_buffer.to_arrow_array::<Float64Type>();
    
    let result = sma::sma_arrow(&input_arrow, opt_period)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
    let len = result.len();
    let buffer = result.values().inner().clone();
    
    Ok(WasmBuffer::from_parts(
        buffer,
        len,
        std::mem::size_of::<f64>(),
    ))
}
