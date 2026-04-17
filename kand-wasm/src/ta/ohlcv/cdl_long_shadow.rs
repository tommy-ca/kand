use crate::WasmBuffer;
use kand::ohlcv::cdl_long_shadow;
use kand::{TAFloat, TAInt};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn cdl_long_shadow(
    input_open: &WasmBuffer,
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    opt_period: usize,
    opt_shadow_factor: TAFloat,
    output_signals: &mut WasmBuffer,
    output_body_avg: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_open.len();
    cdl_long_shadow::cdl_long_shadow(
        input_open.as_slice::<TAFloat>(len),
        input_high.as_slice::<TAFloat>(len),
        input_low.as_slice::<TAFloat>(len),
        input_close.as_slice::<TAFloat>(len),
        opt_period,
        opt_shadow_factor,
        output_signals.as_mut_slice::<TAInt>(len),
        output_body_avg.as_mut_slice::<TAFloat>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub struct CDLLongShadowResult {
    pub signal: TAInt,
    pub body_avg: TAFloat,
}

#[wasm_bindgen]
pub fn cdl_long_shadow_inc(
    input_open: TAFloat,
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    prev_body_avg: TAFloat,
    opt_period: usize,
    opt_shadow_factor: TAFloat,
) -> Result<CDLLongShadowResult, JsValue> {
    cdl_long_shadow::cdl_long_shadow_inc(
        input_open,
        input_high,
        input_low,
        input_close,
        prev_body_avg,
        opt_period,
        opt_shadow_factor,
    )
    .map(|(signal, body_avg)| CDLLongShadowResult { signal, body_avg })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
