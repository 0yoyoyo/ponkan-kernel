#![no_std]
#![no_main]
#![feature(asm)]

use core::mem::MaybeUninit;

#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum PixelFormat {
    kPixelRGBResv8BitPerColor,
    kPixelBGRResv8BitPerColor,
}

#[repr(C)]
pub struct FrameBufferConfig {
    frame_buffer: *mut u8,
    pixels_per_scan_line: u32,
    horisontal_resolution: u32,
    vertical_resolution: u32,
    pixel_format: PixelFormat,
}

struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

trait PixelWriter {
    fn write(&mut self, x: usize, y: usize, color: PixelColor);
    fn pixel_at(&mut self, x: usize, y: usize) -> &mut [u8];
}

struct RGBResv8BitPerColorPixelWriter<'a>(&'a mut FrameBufferConfig);

impl<'a> PixelWriter for RGBResv8BitPerColorPixelWriter<'a> {
    fn write(&mut self, x: usize, y: usize, color: PixelColor) {
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

struct BGRResv8BitPerColorPixelWriter<'a>(&'a mut FrameBufferConfig);

impl<'a> PixelWriter for BGRResv8BitPerColorPixelWriter<'a> {
    fn write(&mut self, x: usize, y: usize, color: PixelColor) {
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

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

static mut BUF_RGB: MaybeUninit<RGBResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_BGR: MaybeUninit<BGRResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();

#[no_mangle]
pub extern "C" fn kernel_main(
    frame_buffer_config: &'static mut FrameBufferConfig,
) -> ! {
    let h_res = frame_buffer_config.horisontal_resolution as usize;
    let v_res = frame_buffer_config.vertical_resolution as usize;

    let pixel_writer: &mut dyn PixelWriter =
        match &frame_buffer_config.pixel_format
    {
        PixelFormat::kPixelRGBResv8BitPerColor => {
            unsafe {
                BUF_RGB.write(
                    RGBResv8BitPerColorPixelWriter(frame_buffer_config));
                BUF_RGB.assume_init_mut()
            }
        },
        PixelFormat::kPixelBGRResv8BitPerColor => {
            unsafe {
                BUF_BGR.write(
                    BGRResv8BitPerColorPixelWriter(frame_buffer_config));
                BUF_BGR.assume_init_mut()
            }
        },
    };

    for x in 0..h_res as usize {
        for y in 0..v_res as usize {
            let color = PixelColor { r: 255, g: 255, b: 255 };
            pixel_writer.write(x, y, color);
        }
    }
    for x in 0..200 {
        for y in 0..100 {
            let color = PixelColor { r: 0, g: 255, b: 0 };
            pixel_writer.write(x, y, color);
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
