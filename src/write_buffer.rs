use core::fmt;

pub struct WriteBuffer<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> fmt::Write for WriteBuffer<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (i, &byte) in s.as_bytes().iter().enumerate() {
            self.buf[self.len + i] = byte;
        }
        self.len += s.len();
        if self.len > self.buf.len() {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

impl<const N: usize> AsRef<str> for WriteBuffer<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> WriteBuffer<N> {
    pub fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buf.fill(0);
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            let s = core::slice::from_raw_parts(
                self.buf.as_ptr(), self.len);
            core::str::from_utf8_unchecked(s)
        }
    }
}
