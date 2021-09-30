#[derive(Debug)]
#[allow(dead_code)]
pub enum PciErrorCode {
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
}

#[derive(Debug)]
pub struct PciError {
    pub code: PciErrorCode,
    pub file: &'static str,
    pub line: u32,
}

#[macro_export]
macro_rules! make_error {
    ($code:expr) => (Err(PciError {
        code: $code,
        file: file!(),
        line: line!(),
    }));
}

pub use crate::make_error;
