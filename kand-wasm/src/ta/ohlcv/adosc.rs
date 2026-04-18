use kand::ta::ohlcv::adosc;
use crate::WasmBuffer;
use crate::ta::types::MAType;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct ADOSCResult {
    pub adosc: f64,
    pub ad: f64,
    pub ad_fast_ema: f64,
    pub ad_slow_ema: f64,
}

/**
 * Returns the lookback period for ADOSC calculation.
 */
#[wasm_bindgen(js_name = adoscLookback)]
pub fn adosc_lookback_wasm(
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_ma_type: MAType,
) -> Result<usize, JsValue> {
    adosc::lookback(opt_fast_period, opt_slow_period, opt_ma_type.into())
        .map(|v| v as usize)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates the Chaikin A/D Oscillator (ADOSC) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = adoscZeroCopy)]
#[allow(clippy::too_many_arguments)]
pub fn adosc_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    input_volume: &WasmBuffer,
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_ma_type: MAType,
    output_adosc: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    adosc::adosc(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        input_volume.as_slice::<f64>(len),
        opt_fast_period,
        opt_slow_period,
        opt_ma_type.into(),
        output_adosc.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates the next ADOSC value incrementally.
 */
#[wasm_bindgen(js_name = adoscInc)]
#[allow(clippy::too_many_arguments)]
pub fn adosc_inc_wasm(
    input_high: f64,
    input_low: f64,
    input_close: f64,
    input_volume: f64,
    prev_ad: f64,
    prev_ad_fast_ema: f64,
    prev_ad_slow_ema: f64,
    opt_fast_period: usize,
    opt_slow_period: usize,
    opt_ma_type: MAType,
) -> Result<ADOSCResult, JsValue> {
    adosc::adosc_inc(
        input_high,
        input_low,
        input_close,
        input_volume,
        prev_ad,
        prev_ad_fast_ema,
        prev_ad_slow_ema,
        opt_fast_period,
        opt_slow_period,
        opt_ma_type.into(),
    )
    .map(|(adosc, ad, ad_fast_ema, ad_slow_ema)| ADOSCResult {
        adosc,
        ad,
        ad_fast_ema,
        ad_slow_ema,
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
