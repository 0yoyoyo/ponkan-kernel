use crate::x86_descriptor::DescriptorType;

use core::ptr::write_volatile;

extern "C" {
    pub fn get_cs() -> u16;
    pub fn load_idt(limit: u16, offset: u64);
}

pub static mut IDT: [InterruptDescriptor; 256] = {
    let empty_interrupt_descriptor = InterruptDescriptor {
        offset_low: 0,
        segment_selector: 0,
        attr: 0,
        offset_middle: 0,
        offset_high: 0,
        reserved: 0,
    };
    [empty_interrupt_descriptor; 256]
};

pub enum InterruptVector {
    Xhci = 0x40,
}

type InterruptDescriptorAttribute = u16;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct InterruptDescriptor  {
    offset_low: u16,
    segment_selector: u16,
    attr: InterruptDescriptorAttribute,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}

pub const fn make_id_attr(
    ty: DescriptorType,
    descriptor_privilege_level: u16,
) -> InterruptDescriptorAttribute {
    make_id_attr_internal(ty, descriptor_privilege_level, true, 0)
}

const fn make_id_attr_internal(
    ty: DescriptorType,
    descriptor_privilege_level: u16,
    present: bool,
    interrupt_stack_table: u16,
) -> InterruptDescriptorAttribute {
    let mut attr: InterruptDescriptorAttribute = 0;
    attr |= interrupt_stack_table & 0b0000_0000_0000_0111;
    attr |= ((ty as u16) << 8) & 0b0000_1111_0000_0000;
    attr |= (descriptor_privilege_level << 13) & 0b0110_0000_0000_0000;
    attr |= ((present as u16) << 15) & 0b1000_0000_0000_0000;
    attr
}

pub fn set_idt_entry(
    desc: &mut InterruptDescriptor,
    attr: InterruptDescriptorAttribute,
    offset: u64,
    segment_selector: u16,
) {
    desc.attr = attr;
    desc.offset_low = (offset & 0x0000_0000_0000_ffff) as u16;
    desc.offset_middle = ((offset >> 16) & 0x0000_0000_0000_ffff) as u16;
    desc.offset_high = ((offset >> 32) & 0x0000_0000_ffff_ffff) as u32;
    desc.segment_selector = segment_selector;
}

pub unsafe fn notify_end_of_interrupt() {
    let end_of_interrupt = 0xfee000b0 as *mut u32;
    write_volatile(end_of_interrupt, 0);
}

#[allow(dead_code)]
#[repr(C)]
pub struct ExceptionStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}
