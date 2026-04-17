use wasm_bindgen::prelude::*;
use std::mem;

pub mod ta;

#[wasm_bindgen]
pub struct WasmBuffer {
    data: Vec<f64>,
}

#[wasm_bindgen]
impl WasmBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(len: usize) -> Self {
        // Simple Vec<f64> is already 8-byte aligned. 
        // For strict 64-byte Arrow alignment, we might need a custom allocator,
        // but for standard f64 arrays in JS, 8-byte is sufficient.
        Self {
            data: vec![0.0; len],
        }
    }

    pub fn ptr(&self) -> *const f64 {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Resizes the buffer. Warning: This may invalidate existing JS views.
    pub fn resize(&mut self, new_len: usize) {
        self.data.resize(new_len, 0.0);
    }
}
