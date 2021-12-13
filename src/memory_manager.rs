use crate::error::*;

use core::mem::size_of;

pub const BYTE_PER_FRAME: usize = 4 * 1024;

pub struct FrameId {
    pub id: usize,
}

impl FrameId {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    #[allow(dead_code)]
    pub fn frame(&mut self) -> *mut u8 {
        (self.id * BYTE_PER_FRAME) as *mut u8
    }
}

// Probably `BitmapMemoryManager` is created in the kernel stack first by
// calling `new`. Therefore, it cannot use over stack size.
const MAX_PHYSICAL_MEMORY_BYTES: usize = 4 * 1024 * 1024 * 1024;
const FRAME_COUNT: usize = MAX_PHYSICAL_MEMORY_BYTES / BYTE_PER_FRAME;

type MapLineType = u32;

const BITS_PER_MAP_LINE: usize = 8 * size_of::<MapLineType>();

pub struct BitmapMemoryManager {
    alloc_map: [MapLineType; FRAME_COUNT / BITS_PER_MAP_LINE],
    range_begin: FrameId,
    range_end: FrameId,
}

impl BitmapMemoryManager {
    pub fn new() -> Self {
        Self {
            alloc_map: [0; FRAME_COUNT / BITS_PER_MAP_LINE],
            range_begin: FrameId::new(0),
            range_end: FrameId::new(FRAME_COUNT),
        }
    }

    pub fn allocate(&mut self, num_frames: usize) -> Result<FrameId, OsError> {
        let mut start_frame_id = self.range_begin.id();
        loop {
            let mut i = 0;
            while i < num_frames {
                if start_frame_id + i >= self.range_end.id() {
                    return make_error!(OsErrorCode::NoEnoughMemory);
                }
                if self.get_bit(FrameId::new(start_frame_id + i)) {
                    break;
                }
                i += 1;
            }
            if i == num_frames {
                self.mark_allocated(FrameId::new(start_frame_id), num_frames);
                return Ok(FrameId::new(start_frame_id));
            }
            start_frame_id += i + 1
        }
    }

    pub fn free(
        &mut self,
        start_frame: FrameId,
        num_frames: usize
    ) -> Result<(), OsError> {
        for i in 0..num_frames {
            self.set_bit(FrameId::new(start_frame.id() + i), false);
        }
        Ok(())
    }

    pub fn mark_allocated(&mut self, start_frame: FrameId, num_frames: usize) {
        for i in 0..num_frames {
            self.set_bit(FrameId::new(start_frame.id() + i), true);
        }
    }

    pub fn set_memory_range(
        &mut self,
        range_begin: FrameId,
        range_end: FrameId
    ) {
        self.range_begin = range_begin;
        self.range_end = range_end;
    }

    fn get_bit(&self, frame: FrameId) -> bool {
        let line_index = frame.id() / BITS_PER_MAP_LINE;
        let bit_index = frame.id() % BITS_PER_MAP_LINE;

        (self.alloc_map[line_index] & (1 << bit_index)) != 0
    }

    fn set_bit(&mut self, frame: FrameId, allocated: bool) {
        let line_index = frame.id() / BITS_PER_MAP_LINE;
        let bit_index = frame.id() % BITS_PER_MAP_LINE;

        if allocated {
            self.alloc_map[line_index] |= 1 << bit_index;
        } else {
            self.alloc_map[line_index] &= !(1 << bit_index);
        }
    }
}
