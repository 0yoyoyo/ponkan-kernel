use crate::graphics::{PixelWriter, PixelColor};

const FONT_A: [u8; 16] = [
    0b00000000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00100100,
    0b00100100,
    0b00100100,
    0b00100100,
    0b01111110,
    0b01000010,
    0b01000010,
    0b01000010,
    0b11100111,
    0b00000000,
    0b00000000,
];

pub fn write_ascii(
    writer: &mut dyn PixelWriter,
    x: usize,
    y: usize,
    c: char,
    color: &PixelColor
) {
    if c != 'A' {
        return;
    }
    for (dy, line) in FONT_A.iter().enumerate() {
        for dx in 0..8 {
            if (line << dx & 0x80u8) == 0x80u8 {
                writer.write(x + dx, y + dy, color);
            }
        }
    }
}

