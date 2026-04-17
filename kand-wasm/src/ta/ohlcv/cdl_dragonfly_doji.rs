use crate::WasmBuffer;
use kand::ohlcv::cdl_dragonfly_doji;
use kand::{TAFloat, TAInt};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn cdl_dragonfly_doji(
    input_open: &WasmBuffer,
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    opt_body_percent: TAFloat,
    opt_shadow_percent: TAFloat,
    output_signals: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_open.len();
    cdl_dragonfly_doji::cdl_dragonfly_doji(
        input_open.as_slice::<TAFloat>(len),
        input_high.as_slice::<TAFloat>(len),
        input_low.as_slice::<TAFloat>(len),
        input_close.as_slice::<TAFloat>(len),
        opt_body_percent,
        opt_shadow_percent,
        output_signals.as_mut_slice::<TAInt>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn cdl_dragonfly_doji_inc(
    input_open: TAFloat,
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    opt_body_percent: TAFloat,
    opt_shadow_percent: TAFloat,
) -> Result<TAInt, JsValue> {
    cdl_dragonfly_doji::cdl_dragonfly_doji_inc(
        input_open,
        input_high,
        input_low,
        input_close,
        opt_body_percent,
        opt_shadow_percent,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
