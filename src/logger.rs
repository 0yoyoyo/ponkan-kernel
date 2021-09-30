#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

pub static mut LOG_LEVEL: LogLevel = Warn;

pub fn set_log_level(level: LogLevel) {
    unsafe {
        LOG_LEVEL = level;
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => ({
        if unsafe { &LOG_LEVEL } >= &($level) {
            let mut buf = WriteBuffer::<1024>::new();
            writeln!(buf, $($arg)*).unwrap();
            let console = unsafe {
                BUF_CONSOLE.assume_init_mut()
            };
            console.put_string(buf);
        }
    });
}

pub use LogLevel::*;

pub use crate::log;
pub use crate::BUF_CONSOLE;
pub use crate::WriteBuffer;

pub use core::fmt::Write;
