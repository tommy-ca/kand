use std::mem;
use wasm_bindgen::prelude::*;

#[cfg(feature = "arrow")]
use arrow_buffer::Buffer;

pub mod ta;

#[wasm_bindgen]
pub struct WasmBuffer {
    #[cfg(feature = "arrow")]
    data: Buffer,
    #[cfg(not(feature = "arrow"))]
    data: Vec<u8>,

    len: usize,
    element_size: usize,
}

#[wasm_bindgen]
impl WasmBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(len: usize, element_size: usize) -> Self {
        let capacity = len * element_size;

        #[cfg(feature = "arrow")]
        {
            let (_, buffer) = kand::helper::buffer_pool::create_pooled_buffer(capacity);
            Self {
                data: buffer,
                len,
                element_size,
            }
        }

        #[cfg(not(feature = "arrow"))]
        {
            Self {
                data: vec![0; capacity],
                len,
                element_size,
            }
        }
    }

    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn element_size(&self) -> usize {
        self.element_size
    }

    pub fn resize(&mut self, new_len: usize) {
        let new_capacity = new_len * self.element_size;

        #[cfg(feature = "arrow")]
        {
            let (_, new_buffer) = kand::helper::buffer_pool::create_pooled_buffer(new_capacity);
            self.data = new_buffer;
        }

        #[cfg(not(feature = "arrow"))]
        {
            self.data.resize(new_capacity, 0);
        }

        self.len = new_len;
    }
}

impl WasmBuffer {
    pub fn as_slice<T>(&self, len: usize) -> &[T] {
        assert!(len <= self.len);
        assert_eq!(mem::size_of::<T>(), self.element_size);
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, len) }
    }

    pub fn as_mut_slice<T>(&mut self, len: usize) -> &mut [T] {
        assert!(len <= self.len);
        assert_eq!(mem::size_of::<T>(), self.element_size);

        #[cfg(feature = "arrow")]
        {
            unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, len) }
        }

        #[cfg(not(feature = "arrow"))]
        {
            unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, len) }
        }
    }

    #[cfg(feature = "arrow")]
    pub fn to_arrow_array<T: arrow::datatypes::ArrowPrimitiveType>(
        &self,
    ) -> arrow::array::PrimitiveArray<T> {
        use arrow::array::PrimitiveArray;
        use arrow_buffer::ScalarBuffer;

        assert_eq!(mem::size_of::<T::Native>(), self.element_size);
        let scalar_buffer = ScalarBuffer::<T::Native>::new(self.data.clone(), 0, self.len);
        PrimitiveArray::<T>::new(scalar_buffer, None)
    }
}

#[cfg(feature = "arrow")]
impl From<Buffer> for WasmBuffer {
    fn from(buffer: Buffer) -> Self {
        let len_bytes = buffer.len();
        Self {
            data: buffer,
            len: len_bytes,
            element_size: 1,
        }
    }
}

#[cfg(feature = "arrow")]
impl WasmBuffer {
    pub fn from_parts(buffer: Buffer, len: usize, element_size: usize) -> Self {
        Self {
            data: buffer,
            len,
            element_size,
        }
    }
}
