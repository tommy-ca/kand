use crate::WasmBuffer;
use kand::ta::ohlcv::sar;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct SARResult {
    pub sar: f64,
    pub is_long: bool,
    pub af: f64,
    pub ep: f64,
}

/**
 * Returns the lookback period for SAR calculation.
 */
#[wasm_bindgen(js_name = sarLookback)]
pub fn sar_lookback_wasm(opt_acceleration: f64, opt_maximum: f64) -> Result<usize, JsValue> {
    sar::lookback(opt_acceleration, opt_maximum).map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Parabolic SAR using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = sarZeroCopy)]
#[allow(clippy::too_many_arguments)]
pub fn sar_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    opt_acceleration: f64,
    opt_maximum: f64,
    output_sar: &mut WasmBuffer,
    output_is_long: &mut WasmBuffer,
    output_af: &mut WasmBuffer,
    output_ep: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    sar::sar(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        opt_acceleration,
        opt_maximum,
        output_sar.as_mut_slice::<f64>(len),
        output_is_long.as_mut_slice::<bool>(len),
        output_af.as_mut_slice::<f64>(len),
        output_ep.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single SAR value incrementally.
 */
#[wasm_bindgen(js_name = sarInc)]
#[allow(clippy::too_many_arguments)]
pub fn sar_inc_wasm(
    input_high: f64,
    input_low: f64,
    prev_high: f64,
    prev_low: f64,
    prev_sar: f64,
    input_is_long: bool,
    input_af: f64,
    input_ep: f64,
    opt_acceleration: f64,
    opt_maximum: f64,
) -> Result<SARResult, JsValue> {
    sar::sar_inc(
        input_high,
        input_low,
        prev_high,
        prev_low,
        prev_sar,
        input_is_long,
        input_af,
        input_ep,
        opt_acceleration,
        opt_maximum,
    )
    .map(|(sar, is_long, af, ep)| SARResult {
        sar,
        is_long,
        af,
        ep,
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
