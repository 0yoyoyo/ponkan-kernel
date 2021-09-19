use crate::graphics::{PixelWriter, PixelColor};

#[link(name="hankaku")]
extern "C" {
    static _binary_hankaku_bin_start: u8;
    static _binary_hankaku_bin_end: u8;
    static _binary_hankaku_bin_size: u8;
}

pub fn get_font(c: char) -> Option<&'static [u8]> {
    const FONT_SIZE: usize = 16;

    if !c.is_ascii() {
        return None;
    }
    let ascii_code = c as u8;
    let index = FONT_SIZE * (ascii_code as usize);

    unsafe {
        let font_list_size =
            &_binary_hankaku_bin_size as *const u8 as usize;
        if index >= font_list_size {
            return None;
        }

        let font_list_start =
            &_binary_hankaku_bin_start as *const u8;
        Some(core::slice::from_raw_parts(
                font_list_start.add(index), FONT_SIZE))
    }
}

pub fn write_ascii(
    writer: &mut dyn PixelWriter,
    x: usize,
    y: usize,
    c: char,
    color: &PixelColor
) {
    if let Some(font) = get_font(c) {
        for (dy, line) in font.iter().enumerate() {
            for dx in 0..8 {
                if (line << dx & 0x80u8) == 0x80u8 {
                    writer.write(x + dx, y + dy, color);
                }
            }
        }
    }
}

pub fn write_string(
    writer: &mut dyn PixelWriter,
    x: usize,
    y: usize,
    s: &str,
    color: &PixelColor
) {
    const FONT_WIDTH: usize = 8;
    for (i, c) in s.chars().enumerate() {
        write_ascii(writer, x + FONT_WIDTH * i, y, c, color);
    }
}
