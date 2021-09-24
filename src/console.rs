use crate::font::{
    FONT_WIDTH, FONT_HEIGHT,
    write_ascii, write_string
};
use crate::graphics::{PixelColor, PixelWriter};

const ROWS: usize = 25;
const COLUMNS: usize = 80;

pub struct Console<'a> {
    buffer: [[u8; COLUMNS]; ROWS],
    cursor_row: usize,
    cursor_column: usize,
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
            buffer: [[0; COLUMNS]; ROWS],
            cursor_row: 0,
            cursor_column: 0,
            fg_color,
            bg_color,
            writer,
        }
    }
}

impl<'a> Console<'a> {
    pub fn put_string<A: AsRef<str>>(&mut self, s: A) {
        for c in s.as_ref().chars() {
            if c == '\n' {
                self.new_line();
            } else if self.cursor_column < COLUMNS {
                write_ascii(
                    self.writer,
                    FONT_WIDTH * self.cursor_column,
                    FONT_HEIGHT * self.cursor_row,
                    c,
                    &self.fg_color,
                );
                self.buffer[self.cursor_row][self.cursor_column] = c as u8;
                self.cursor_column += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            for y in 0..(FONT_HEIGHT * ROWS) {
                for x in 0..(FONT_WIDTH * COLUMNS) {
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
                    FONT_HEIGHT * row,
                    s,
                    &self.fg_color
                );
            }
            self.buffer[ROWS - 1].fill(0);
        }
    }
}
