use crate::graphics::{
    PixelColor, PixelWriter, Vector2D, Displacement
};

use core::cell::RefCell;

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

pub struct MouseCursor<'a> {
    writer: &'a RefCell<&'a mut dyn PixelWriter>,
    erase_color: PixelColor,
    position: Vector2D,
}

impl<'a> MouseCursor<'a> {
    pub fn new(
        writer: &'a RefCell<&'a mut dyn PixelWriter>,
        erase_color: PixelColor,
        initial_position: Vector2D,
    ) -> Self {
        let this = Self {
            writer,
            erase_color,
            position: initial_position,
        };
        draw_mouse_cursor(this.writer, &this.position);
        this
    }

    pub fn move_relative(&mut self, displacement: Displacement) {
        erase_mouse_cursor(self.writer, &self.erase_color, &self.position);
        self.position.displace(&displacement);
        draw_mouse_cursor(self.writer, &self.position);
    }
}

fn draw_mouse_cursor(
    writer: &RefCell<&mut dyn PixelWriter>,
    position: &Vector2D,
) {
    for (y, &row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel == b'@' {
                let black = PixelColor { r: 0, g: 0, b: 0 };
                writer.borrow_mut().write(
                    position.x + x,
                    position.y + y,
                    &black,
                );
            } else if pixel == b'.' {
                let white = PixelColor { r: 255, g: 255, b: 255 };
                writer.borrow_mut().write(
                    position.x + x,
                    position.y + y,
                    &white,
                );
            }
        }
    }
}

fn erase_mouse_cursor(
    writer: &RefCell<&mut dyn PixelWriter>,
    erase_color: &PixelColor,
    position: &Vector2D,
) {
    for (y, &row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel != b' ' {
                writer.borrow_mut().write(
                    position.x + x,
                    position.y + y,
                    erase_color,
                );
            }
        }
    }
}
