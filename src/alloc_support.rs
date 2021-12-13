use crate::BUF_MEMMNG;
use crate::memory_manager::{FrameId, BYTE_PER_FRAME};

use core::{alloc::GlobalAlloc, alloc::Layout, ptr::null_mut};

struct MemoryAllocator;

unsafe impl GlobalAlloc for MemoryAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let memory_manager = BUF_MEMMNG.assume_init_mut();
        let num_frames = (layout.size() + BYTE_PER_FRAME - 1) / BYTE_PER_FRAME;
        match memory_manager.allocate(num_frames) {
            Ok(frame) => (frame.id() * BYTE_PER_FRAME) as *mut u8,
            Err(_) => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let memory_manager = BUF_MEMMNG.assume_init_mut();
        let num_frames = (layout.size() + BYTE_PER_FRAME - 1) / BYTE_PER_FRAME;
        let start_frame = FrameId::new(ptr as usize / BYTE_PER_FRAME);
        memory_manager.free(start_frame, num_frames).unwrap();
    }
}

#[global_allocator]
static ALLOCATOR: MemoryAllocator = MemoryAllocator;
