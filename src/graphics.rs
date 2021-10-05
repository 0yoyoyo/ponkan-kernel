use crate::frame_buffer_config::FrameBufferConfig;

use core::cell::RefCell;

#[derive(Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Vector2D {
    pub x: usize,
    pub y: usize,
}

pub struct Displacement {
    pub x: isize,
    pub y: isize,
}

impl Vector2D {
    pub fn displace(&mut self, displacement: &Displacement) {
        self.x = if displacement.x >= 0 {
            self.x.saturating_add(displacement.x.abs() as usize)
        } else {
            self.x.saturating_sub(displacement.x.abs() as usize)
        };
        self.y = if displacement.y >= 0 {
            self.y.saturating_add(displacement.y.abs() as usize)
        } else {
            self.y.saturating_sub(displacement.y.abs() as usize)
        };
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
