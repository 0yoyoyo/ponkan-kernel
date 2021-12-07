#[derive(Debug)]
#[allow(dead_code)]
pub enum OsErrorCode {
    Full,
    Empty,
    NoEnoughMemory,
    IndexOutOfRange,
    HostControllerNotHalted,
    InvalidSlotId,
    PortNotConnected,
    InvalidEndpointNumber,
    TransferRingNotSet,
    AlreadyAllocated,
    NotImplemented,
    InvalidDescriptor,
    BufferTooSmall,
    UnknownDevice,
    NoCorrespondingSetupStage,
    TransferFailed,
    InvaludPhase,
    UnknownXhciSpeedId,
    NoWaiter,
    NoPciMsi,
}

#[derive(Debug)]
pub struct OsError {
    pub code: OsErrorCode,
    pub file: &'static str,
    pub line: u32,
}

#[macro_export]
macro_rules! make_error {
    ($code:expr) => (Err(OsError {
        code: $code,
        file: file!(),
        line: line!(),
    }));
}

pub use crate::make_error;
