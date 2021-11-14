#[derive(Clone, Copy)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum PixelFormat {
    kPixelRGBResv8BitPerColor,
    kPixelBGRResv8BitPerColor,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrameBufferConfig {
    pub frame_buffer: *mut u8,
    pub pixels_per_scan_line: u32,
    pub horisontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: PixelFormat,
}
