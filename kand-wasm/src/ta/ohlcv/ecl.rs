use kand::ta::ohlcv::ecl;
use crate::WasmBuffer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct ECLResult {
    pub h5: f64,
    pub h4: f64,
    pub h3: f64,
    pub h2: f64,
    pub h1: f64,
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub l4: f64,
    pub l5: f64,
}

/**
 * Calculates Expanded Camarilla Levels (ECL) using zero-copy WasmBuffers.
 */
#[wasm_bindgen(js_name = eclZeroCopy)]
#[allow(clippy::too_many_arguments)]
pub fn ecl_wasm_zero_copy(
    input_high: &WasmBuffer,
    input_low: &WasmBuffer,
    input_close: &WasmBuffer,
    output_h5: &mut WasmBuffer,
    output_h4: &mut WasmBuffer,
    output_h3: &mut WasmBuffer,
    output_h2: &mut WasmBuffer,
    output_h1: &mut WasmBuffer,
    output_l1: &mut WasmBuffer,
    output_l2: &mut WasmBuffer,
    output_l3: &mut WasmBuffer,
    output_l4: &mut WasmBuffer,
    output_l5: &mut WasmBuffer,
) -> Result<(), JsValue> {
    let len = input_high.len();
    ecl::ecl(
        input_high.as_slice::<f64>(len),
        input_low.as_slice::<f64>(len),
        input_close.as_slice::<f64>(len),
        output_h5.as_mut_slice::<f64>(len),
        output_h4.as_mut_slice::<f64>(len),
        output_h3.as_mut_slice::<f64>(len),
        output_h2.as_mut_slice::<f64>(len),
        output_h1.as_mut_slice::<f64>(len),
        output_l1.as_mut_slice::<f64>(len),
        output_l2.as_mut_slice::<f64>(len),
        output_l3.as_mut_slice::<f64>(len),
        output_l4.as_mut_slice::<f64>(len),
        output_l5.as_mut_slice::<f64>(len),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

/**
 * Incrementally calculates ECL for a single period.
 */
#[wasm_bindgen(js_name = eclInc)]
pub fn ecl_wasm_inc(
    prev_high: f64,
    prev_low: f64,
    prev_close: f64,
) -> Result<ECLResult, JsValue> {
    ecl::ecl_inc(prev_high, prev_low, prev_close)
        .map(|(h5, h4, h3, h2, h1, l1, l2, l3, l4, l5)| ECLResult {
            h5,
            h4,
            h3,
            h2,
            h1,
            l1,
            l2,
            l3,
            l4,
            l5,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
