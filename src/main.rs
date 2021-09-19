#![no_std]
#![no_main]
#![feature(asm)]

mod panic;
mod graphics;
mod font;
mod frame_buffer_config;
mod write_buffer;

use graphics::{
    PixelColor, PixelWriter,
    RGBResv8BitPerColorPixelWriter,
    BGRResv8BitPerColorPixelWriter,
};
use font::{write_ascii, write_string};
use frame_buffer_config::{
    PixelFormat, FrameBufferConfig
};
use write_buffer::WriteBuffer;

use core::{fmt::Write, mem::MaybeUninit};

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
            let white = PixelColor { r: 255, g: 255, b: 255 };
            pixel_writer.write(x, y, &white);
        }
    }
    for x in 0..200 {
        for y in 0..100 {
            let green = PixelColor { r: 0, g: 255, b: 0 };
            pixel_writer.write(x, y, &green);
        }
    }

    let black = PixelColor { r: 0, g: 0, b: 0 };
    write_ascii(pixel_writer, 50, 50, 'A', &black);
    write_ascii(pixel_writer, 58, 50, 'z', &black);
    write_ascii(pixel_writer, 66, 50, '?', &black);

    let blue = PixelColor { r: 0, g: 0, b: 255 };
    write_string(pixel_writer, 0, 66, "Hello, world!", &blue);
    let mut buf = WriteBuffer::<128>::new();
    write!(buf, "1 + 2 = {}", 1 + 2).unwrap();
    write_string(pixel_writer, 0, 82, buf.as_str(), &blue);

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
