use crate::frame_buffer_config::FrameBufferConfig;

use core::{cell::RefCell, ops::AddAssign};

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Vector2D {
    pub x: usize,
    pub y: usize,
}

impl AddAssign for Vector2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
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

pub fn fill_rectangle(
    writer: &RefCell<&mut dyn PixelWriter>,
    pos: &Vector2D,
    size: &Vector2D,
    color: &PixelColor,
) {
    for y in 0..size.y {
        for x in 0..size.x {
            writer.borrow_mut().write(pos.x + x, pos.y + y, color);
        }
    }
}

pub fn draw_rectangle(
    writer: &RefCell<&mut dyn PixelWriter>,
    pos: &Vector2D,
    size: &Vector2D,
    color: &PixelColor,
) {
    for x in 0..size.x {
        writer.borrow_mut().write(pos.x + x, pos.y, color);
        writer.borrow_mut().write(pos.x + x, pos.y + size.y - 1, color);
    }
    for y in 0..size.y {
        writer.borrow_mut().write(pos.x, pos.y + y, color);
        writer.borrow_mut().write(pos.x + size.x - 1, pos.y + y, color);
    }
}
