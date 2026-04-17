use crate::WasmBuffer;
use kand::ohlcv::cdl_marubozu;
use kand::{TAFloat, TAInt};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn cdl_marubozu(
    input_open: &WasmBuffer,
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    opt_period: usize,
    opt_shadow_percent: TAFloat,
    output_signals: &mut WasmBuffer,
    output_body_avg: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_open.len();
    cdl_marubozu::cdl_marubozu(
        input_open.as_slice::<TAFloat>(len),
        input_high.as_slice::<TAFloat>(len),
        input_low.as_slice::<TAFloat>(len),
        input_close.as_slice::<TAFloat>(len),
        opt_period,
        opt_shadow_percent,
        output_signals.as_mut_slice::<TAInt>(len),
        output_body_avg.as_mut_slice::<TAFloat>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub struct CDLMarubozuResult {
    pub signal: TAInt,
    pub body_avg: TAFloat,
}

#[wasm_bindgen]
pub fn cdl_marubozu_inc(
    input_open: TAFloat,
    input_high: TAFloat,
    input_low: TAFloat,
    input_close: TAFloat,
    prev_body_avg: TAFloat,
    opt_period: usize,
    opt_shadow_percent: TAFloat,
) -> Result<CDLMarubozuResult, JsValue> {
    cdl_marubozu::cdl_marubozu_inc(
        input_open,
        input_high,
        input_low,
        input_close,
        prev_body_avg,
        opt_period,
        opt_shadow_percent,
    )
    .map(|(signal, body_avg)| CDLMarubozuResult { signal, body_avg })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
