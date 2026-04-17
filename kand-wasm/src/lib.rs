use wasm_bindgen::prelude::*;
use std::mem;

pub mod ta;

#[wasm_bindgen]
pub struct WasmBuffer {
    data: Vec<u8>,
    len: usize,
    element_size: usize,
}

#[wasm_bindgen]
impl WasmBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(len: usize, element_size: usize) -> Self {
        Self {
            data: vec![0; len * element_size],
            len,
            element_size,
        }
    }

    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn element_size(&self) -> usize {
        self.element_size
    }

    /// Resizes the buffer. Warning: This may invalidate existing JS views.
    pub fn resize(&mut self, new_len: usize) {
        self.data.resize(new_len * self.element_size, 0);
        self.len = new_len;
    }
}

impl WasmBuffer {
    pub fn as_slice<T>(&self, len: usize) -> &[T] {
        assert!(len <= self.len);
        assert_eq!(mem::size_of::<T>(), self.element_size);
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const T, len)
        }
    }

    pub fn as_mut_slice<T>(&mut self, len: usize) -> &mut [T] {
        assert!(len <= self.len);
        assert_eq!(mem::size_of::<T>(), self.element_size);
        unsafe {
            std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, len)
        }
    }
}
