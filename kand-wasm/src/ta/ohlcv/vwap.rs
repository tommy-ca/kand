use crate::WasmBuffer;
use kand::ta::ohlcv::vwap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct VWAPResult {
    pub vwap: f64,
    pub cum_pv: f64,
    pub cum_vol: f64,
}

/**
 * Returns the lookback period for VWAP calculation.
 */
#[wasm_bindgen(js_name = vwapLookback)]
pub fn vwap_lookback_wasm() -> Result<usize, JsValue> {
    vwap::lookback()
        .map(|v| v as usize)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Volume Weighted Average Price (VWAP) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = vwapZeroCopy)]
#[allow(clippy::too_many_arguments)]
pub fn vwap_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    input_volume: &WasmBuffer,
    output_vwap: &mut WasmBuffer,
    output_cum_pv: &mut WasmBuffer,
    output_cum_vol: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    vwap::vwap(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        input_volume.as_slice::<f64>(len),
        output_vwap.as_mut_slice::<f64>(len),
        output_cum_pv.as_mut_slice::<f64>(len),
        output_cum_vol.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single VWAP value incrementally.
 */
#[wasm_bindgen(js_name = vwapInc)]
pub fn vwap_inc_wasm(
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    prev_cum_pv: f64,
    prev_cum_vol: f64,
) -> Result<VWAPResult, JsValue> {
    vwap::vwap_inc(high, low, close, volume, prev_cum_pv, prev_cum_vol)
        .map(|(vwap, cum_pv, cum_vol)| VWAPResult {
            vwap,
            cum_pv,
            cum_vol,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
