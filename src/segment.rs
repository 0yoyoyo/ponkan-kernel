use crate::x86_descriptor::SegmentDescriptorType;

extern "C" {
    fn load_gdt(limit: u16, offset: u64);
}

static mut GDT: [SegmentDescriptor; 3] = {
    let empty_desc = SegmentDescriptor {
        data: 0,
    };
    [empty_desc; 3]
};

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct SegmentDescriptorFields {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    others1: u8,
    others2: u8,
    base_high: u8,
}

#[derive(Clone, Copy)]
union SegmentDescriptor {
    data: u64,
    fields: SegmentDescriptorFields,
}

fn set_code_segment(
    desc: &mut SegmentDescriptor,
    desc_type: SegmentDescriptorType,
    desc_privilege_level: u8,
    base: u32,
    limit: u32,
) {
    desc.data = 0;

    desc.fields.base_low = (base & 0x0000ffff) as u16;
    desc.fields.base_middle = ((base >> 16) & 0x000000ff) as u8;
    desc.fields.base_high = ((base >> 24) & 0x000000ff) as u8;

    desc.fields.limit_low = (limit & 0x0000ffff) as u16;

    unsafe {
        desc.fields.others1 |= (desc_type as u8) & 0b00001111;
        desc.fields.others1 |= 1 << 4; // system_segment
        desc.fields.others1 |= (desc_privilege_level & 0b00000011) >> 5;
        desc.fields.others1 |= 1 << 7; // present

        desc.fields.others2 |= ((limit >> 16) & 0x0000000f) as u8;
        desc.fields.others2 |= 0 << 4; // available
        desc.fields.others2 |= 1 << 5; // long_mode
        desc.fields.others2 |= 0 << 6; // default_operation_size
        desc.fields.others2 |= 1 << 7; // granularity
    }
}

fn set_data_segment(
    desc: &mut SegmentDescriptor,
    desc_type: SegmentDescriptorType,
    desc_privilege_level: u8,
    base: u32,
    limit: u32,
) {
    set_code_segment(desc, desc_type, desc_privilege_level, base, limit);

    unsafe {
        desc.fields.others2 &= 0b10011111;
        desc.fields.others2 |= 0 << 5; // long_mode
        desc.fields.others2 |= 1 << 6; // default_operation_size
    }
}

pub fn setup_segments() {
    use SegmentDescriptorType::*;
    unsafe {
        GDT[0].data = 0;
        set_code_segment(&mut GDT[1], ExecuteRead, 0, 0, 0x000fffff);
        set_data_segment(&mut GDT[2], ReadWrite, 0, 0, 0x000fffff);
        load_gdt(
            (core::mem::size_of_val(&GDT) - 1) as u16,
            &GDT as *const _ as u64
        );
    }
}
