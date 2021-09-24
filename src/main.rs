#![no_std]
#![no_main]
#![feature(asm)]

mod panic;
mod graphics;
mod font;
mod frame_buffer_config;
mod write_buffer;
mod console;

use graphics::{
    PixelColor, PixelWriter, Vector2D,
    RGBResv8BitPerColorPixelWriter,
    BGRResv8BitPerColorPixelWriter,
    fill_rectangle, draw_rectangle,
};
use frame_buffer_config::{
    PixelFormat, FrameBufferConfig
};
use write_buffer::WriteBuffer;
use console::Console;

use core::{cell::RefCell, fmt::Write, mem::MaybeUninit};

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
const MOUSE_CURSOR_SHAPE: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
    *b"@              ",
    *b"@@             ",
    *b"@.@            ",
    *b"@..@           ",
    *b"@...@          ",
    *b"@....@         ",
    *b"@.....@        ",
    *b"@......@       ",
    *b"@.......@      ",
    *b"@........@     ",
    *b"@.........@    ",
    *b"@..........@   ",
    *b"@...........@  ",
    *b"@............@ ",
    *b"@......@@@@@@@@",
    *b"@......@       ",
    *b"@....@@.@      ",
    *b"@...@ @.@      ",
    *b"@..@   @.@     ",
    *b"@.@    @.@     ",
    *b"@@      @.@    ",
    *b"@       @.@    ",
    *b"         @.@   ",
    *b"         @@@   ",
];

static mut BUF_RGB: MaybeUninit<RGBResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_BGR: MaybeUninit<BGRResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_WRITER: MaybeUninit<RefCell<&mut dyn PixelWriter>> =
    MaybeUninit::uninit();
static mut BUF_CONSOLE: MaybeUninit<Console> = MaybeUninit::uninit();

macro_rules! _kprint {
    ($w:ident, $($arg:tt)*) => ({
        let mut buf = WriteBuffer::<1024>::new();
        $w!(buf, $($arg)*).unwrap();
        let console = unsafe {
            BUF_CONSOLE.assume_init_mut()
        };
        console.put_string(buf);
    });
}

#[allow(unused_macros)]
macro_rules! kprint {
    ($($arg:tt)*) => (_kprint!(write, $($arg)*));
}

#[allow(unused_macros)]
macro_rules! kprintln {
    () => (_kprint!(write, "\n"));
    ($($arg:tt)*) => (_kprint!(writeln, $($arg)*));
}

#[no_mangle]
pub extern "C" fn kernel_main(
    frame_buffer_config: &'static mut FrameBufferConfig,
) -> ! {
    let frame_width = frame_buffer_config.horisontal_resolution as usize;
    let frame_height = frame_buffer_config.vertical_resolution as usize;

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
    let pixel_writer = unsafe {
        BUF_WRITER.write(RefCell::new(pixel_writer));
        BUF_WRITER.assume_init_ref()
    };

    let desktop_bg_color = PixelColor { r: 45, g: 115, b: 200 };
    let desktop_fg_color = PixelColor { r: 255, g: 255, b: 255 };
    fill_rectangle(
        pixel_writer,
        &Vector2D { x: 0, y: 0 },
        &Vector2D { x: frame_width, y: frame_height - 40 },
        &desktop_bg_color,
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D { x: 0, y: frame_height - 40 },
        &Vector2D { x: frame_width, y: 40 },
        &PixelColor { r: 10, g: 30, b: 50 },
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D { x: 0, y: frame_height - 40 },
        &Vector2D { x: frame_width / 4, y: 40 },
        &PixelColor { r: 120, g: 120, b: 120 },
    );
    draw_rectangle(
        pixel_writer,
        &Vector2D { x: 10, y: frame_height - 30 },
        &Vector2D { x: 20, y: 20 },
        &PixelColor { r: 50, g: 160, b: 50 },
    );

    unsafe {
        BUF_CONSOLE.write(
            Console::new(desktop_fg_color, desktop_bg_color, pixel_writer)
        );
    }
    kprintln!("Welcome to PonkanOS!");

    for (y, &row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel == b'@' {
                let black = PixelColor { r: 0, g: 0, b: 0 };
                pixel_writer.borrow_mut().write(200 + x, 100 + y, &black);
            } else if pixel == b'.' {
                let white = PixelColor { r: 255, g: 255, b: 255 };
                pixel_writer.borrow_mut().write(200 + x, 100 + y, &white);
            }
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
