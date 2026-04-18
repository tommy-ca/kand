use crate::WasmBuffer;
use kand::ta::ohlcv::vegas;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct VegasResult {
    pub channel_upper: f64,
    pub channel_lower: f64,
    pub boundary_upper: f64,
    pub boundary_lower: f64,
}

/**
 * Returns the lookback period for VEGAS calculation.
 */
#[wasm_bindgen(js_name = vegasLookback)]
pub fn vegas_lookback_wasm() -> Result<usize, JsValue> {
    vegas::lookback()
        .map(|v| v as usize)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates VEGAS (Volume and EMA Guided Adaptive Scaling) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = vegasZeroCopy)]
#[allow(clippy::too_many_arguments)]
pub fn vegas_wasm_zero_copy(
    input_price: &WasmBuffer,
    output_channel_upper: &mut WasmBuffer,
    output_channel_lower: &mut WasmBuffer,
    output_boundary_upper: &mut WasmBuffer,
    output_boundary_lower: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_price.len();
    vegas::vegas(
        input_price.as_slice::<f64>(len),
        output_channel_upper.as_mut_slice::<f64>(len),
        output_channel_lower.as_mut_slice::<f64>(len),
        output_boundary_upper.as_mut_slice::<f64>(len),
        output_boundary_lower.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single VEGAS value incrementally.
 */
#[wasm_bindgen(js_name = vegasInc)]
pub fn vegas_inc_wasm(
    input_price: f64,
    prev_channel_upper: f64,
    prev_channel_lower: f64,
    prev_boundary_upper: f64,
    prev_boundary_lower: f64,
) -> Result<VegasResult, JsValue> {
    vegas::vegas_inc(
        input_price,
        prev_channel_upper,
        prev_channel_lower,
        prev_boundary_upper,
        prev_boundary_lower,
    )
    .map(
        |(channel_upper, channel_lower, boundary_upper, boundary_lower)| VegasResult {
            channel_upper,
            channel_lower,
            boundary_upper,
            boundary_lower,
        },
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
