use crate::font::{
    FONT_WIDTH, FONT_HIGHT,
    write_ascii, write_string
};
use crate::graphics::{PixelColor, PixelWriter};

const ROWS: usize = 25;
const COLUMS: usize = 80;

pub struct Console<'a> {
    buffer: [[u8; COLUMS]; ROWS],
    cursor_row: usize,
    cursor_colum: usize,
    fg_color: PixelColor,
    bg_color: PixelColor,
    writer: &'a mut dyn PixelWriter,
}

impl<'a> Console<'a> {
    pub fn new(
        fg_color: PixelColor,
        bg_color: PixelColor,
        writer: &'a mut dyn PixelWriter,
    ) -> Self {
        Console {
            buffer: [[0; COLUMS]; ROWS],
            cursor_row: 0,
            cursor_colum: 0,
            fg_color,
            bg_color,
            writer,
        }
    }
}

impl<'a> Console<'a> {
    pub fn put_string(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.new_line();
            } else if self.cursor_colum < COLUMS {
                write_ascii(
                    self.writer,
                    FONT_WIDTH * self.cursor_colum,
                    FONT_HIGHT * self.cursor_row,
                    c,
                    &self.fg_color,
                );
                self.buffer[self.cursor_row][self.cursor_colum] = c as u8;
                self.cursor_colum += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_colum = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            for y in 0..(FONT_HIGHT * ROWS) {
                for x in 0..(FONT_WIDTH * COLUMS) {
                    self.writer.write(x, y, &self.bg_color);
                }
            }
            for row in 0..(ROWS - 1) {
                self.buffer.copy_within((row + 1)..(row + 2), row);
                let s = unsafe {
                    // Initial values at the back of buffer are also
                    // included in string.
                    core::str::from_utf8_unchecked(&self.buffer[row])
                };
                write_string(
                    self.writer,
                    0,
                    FONT_HIGHT * row,
                    s,
                    &self.fg_color
                );
            }
            self.buffer[ROWS - 1].fill(0);
        }
    }
}
