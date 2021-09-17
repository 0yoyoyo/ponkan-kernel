use crate::frame_buffer_config::FrameBufferConfig;

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait PixelWriter {
    fn write(&mut self, x: usize, y: usize, color: &PixelColor);
    fn pixel_at(&mut self, x: usize, y: usize) -> &mut [u8];
}

pub struct RGBResv8BitPerColorPixelWriter<'a>(pub &'a mut FrameBufferConfig);

impl<'a> PixelWriter for RGBResv8BitPerColorPixelWriter<'a> {
    fn write(&mut self, x: usize, y: usize, color: &PixelColor) {
        let pixel = self.pixel_at(x, y);
        pixel[0] = color.r;
        pixel[1] = color.b;
        pixel[2] = color.g;
    }

    fn pixel_at(&mut self, x: usize, y: usize) -> &mut [u8] {
        unsafe {
            const PIXEL_SIZE: usize = 4;
            let p = self.0.frame_buffer.add(
                PIXEL_SIZE * ((self.0.pixels_per_scan_line as usize) * y + x)
            );
            core::slice::from_raw_parts_mut(p, PIXEL_SIZE)
        }
    }
}

pub struct BGRResv8BitPerColorPixelWriter<'a>(pub &'a mut FrameBufferConfig);

impl<'a> PixelWriter for BGRResv8BitPerColorPixelWriter<'a> {
    fn write(&mut self, x: usize, y: usize, color: &PixelColor) {
        let pixel = self.pixel_at(x, y);
        pixel[0] = color.b;
        pixel[1] = color.g;
        pixel[2] = color.r;
    }

    fn pixel_at(&mut self, x: usize, y: usize) -> &mut [u8] {
        unsafe {
            const PIXEL_SIZE: usize = 4;
            let p = self.0.frame_buffer.add(
                PIXEL_SIZE * ((self.0.pixels_per_scan_line as usize) * y + x)
            );
            core::slice::from_raw_parts_mut(p, PIXEL_SIZE)
        }
    }
}
