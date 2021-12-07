use crate::error::*;

pub struct ArrayQueue<'a, T, const N: usize> {
    data: &'a mut [T; N],
    read_pos: usize,
    write_pos: usize,
    count: usize,
    capacity: usize,
}

impl<'a, T, const N: usize> ArrayQueue<'a, T, N> {
    pub fn new(data: &'a mut [T; N]) -> Self {
        Self {
            data,
            read_pos: 0,
            write_pos: 0,
            count: 0,
            capacity: N,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), OsError> {
        if self.count == self.capacity {
            return make_error!(OsErrorCode::Full);
        }

        self.data[self.write_pos] = value;
        self.count += 1;
        self.write_pos += 1;
        if self.write_pos == self.capacity {
            self.write_pos = 0;
        }

        Ok(())
    }

    pub fn pop(&mut self) -> Result<&T, OsError> {
        if self.count == 0 {
            return make_error!(OsErrorCode::Empty);
        }

        let value = &self.data[self.read_pos];
        self.count -= 1;
        self.read_pos += 1;
        if self.read_pos == self.capacity {
            self.read_pos = 0;
        }
        Ok(value)
    }

    pub fn count(&mut self) -> usize {
        self.count
    }
}
