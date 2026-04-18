use crate::WasmBuffer;
use kand::ta::ohlcv::trix;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct TRIXResult {
    pub trix: f64,
    pub ema1: f64,
    pub ema2: f64,
    pub ema3: f64,
}

/**
 * Returns the lookback period for TRIX calculation.
 */
#[wasm_bindgen(js_name = trixLookback)]
pub fn trix_lookback_wasm(opt_period: usize) -> Result<usize, JsValue> {
    trix::lookback(opt_period)
        .map(|v| v as usize)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates Triple Exponential Moving Average Oscillator (TRIX) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = trixZeroCopy)]
pub fn trix_wasm_zero_copy(
    input: &WasmBuffer,
    opt_period: usize,
    output: &mut WasmBuffer,
    ema1_output: &mut WasmBuffer,
    ema2_output: &mut WasmBuffer,
    ema3_output: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input.len();
    trix::trix(
        input.as_slice::<f64>(len),
        opt_period,
        output.as_mut_slice::<f64>(len),
        ema1_output.as_mut_slice::<f64>(len),
        ema2_output.as_mut_slice::<f64>(len),
        ema3_output.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Calculates a single TRIX value incrementally.
 */
#[wasm_bindgen(js_name = trixInc)]
pub fn trix_inc_wasm(
    input: f64,
    prev_ema1: f64,
    prev_ema2: f64,
    prev_ema3: f64,
    opt_period: usize,
) -> Result<TRIXResult, JsValue> {
    trix::trix_inc(input, prev_ema1, prev_ema2, prev_ema3, opt_period)
        .map(|(trix, ema1, ema2, ema3)| TRIXResult {
            trix,
            ema1,
            ema2,
            ema3,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
