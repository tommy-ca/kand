use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::sync::Arc;
use arrow_buffer::Buffer;

/// Standard alignment for Arrow buffers.
const ALIGNMENT: usize = 64;

/// A block of memory managed by our pool.
struct Block {
    ptr: NonNull<u8>,
    layout: Layout,
}

unsafe impl Send for Block {}
unsafe impl Sync for Block {}

impl Block {
    fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, ALIGNMENT).unwrap();
        let ptr = unsafe { alloc(layout) };
        Self {
            ptr: NonNull::new(ptr).expect("Allocation failed"),
            layout,
        }
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr.as_ptr(), self.layout) };
    }
}

/// A custom Allocation that returns its block to a pool on drop.
struct PooledAllocation {
    block: Option<Block>,
}

// In Arrow 58.1, Allocation is blanket implemented for RefUnwindSafe + Send + Sync.
// We must ensure PooledAllocation satisfies these bounds.
impl std::panic::RefUnwindSafe for PooledAllocation {}

impl Drop for PooledAllocation {
    fn drop(&mut self) {
        if let Some(block) = self.block.take() {
            release_block(block);
        }
    }
}

#[cfg(feature = "arrow")]
thread_local! {
    /// A thread-local cache of memory blocks to avoid frequent allocations.
    static BLOCK_CACHE: std::cell::RefCell<Vec<Block>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Acquires a block of memory of at least the given capacity.
fn acquire_block(capacity: usize) -> Block {
    BLOCK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(pos) = cache.iter().position(|b| b.layout.size() >= capacity) {
            cache.swap_remove(pos)
        } else {
            Block::new(capacity)
        }
    })
}

/// Releases a block of memory back to the cache.
fn release_block(block: Block) {
    BLOCK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() < 16 {
            cache.push(block);
        }
    });
}

/// Creates an Arrow Buffer from a pooled allocation.
///
/// This provides zero-copy interoperability while reusing underlying heap memory.
#[cfg(feature = "arrow")]
pub fn create_pooled_buffer(len_bytes: usize) -> (*mut u8, Buffer) {
    let block = acquire_block(len_bytes);
    let ptr = block.ptr.as_ptr();
    
    let allocation = PooledAllocation { block: Some(block) };
    let buffer = unsafe {
        Buffer::from_custom_allocation(
            NonNull::new(ptr).unwrap(),
            len_bytes,
            Arc::new(allocation),
        )
    };
    
    (ptr, buffer)
}
