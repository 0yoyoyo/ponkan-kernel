#[allow(dead_code)]
pub enum DescriptorType {
    Upper8Bytes   = 0,
    Ldt           = 2,
    TssAvailable  = 9,
    TssBusy       = 11,
    CallGate      = 12,
    InterruptGate = 14,
    TrapGate      = 15,
}

#[repr(u8)]
pub enum SegmentDescriptorType {
    ReadWrite   = 2,
    ExecuteRead = 10,
}
