use core::convert::TryFrom;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MemoryMap {
    buffer_size: u64,
    buffer: *const u8,
    map_size: u64,
    map_key: u64,
    descriptor_size: u64,
    descriptor_version: u32,
}

#[repr(C)]
pub struct MemoryDescriptor {
    pub memory_type: u32,
    pub physical_start: usize,
    pub virtual_start: usize,
    pub number_of_pages: u64,
    pub attribute: u64,
}

pub struct Iter<'a> {
    map: &'a MemoryMap,
    cur: usize,
}

impl MemoryMap {
    pub fn iter(&self) -> Iter {
        Iter {
            map: self,
            cur: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a MemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let p = self.map.buffer as usize + self.cur;
        if p < self.map.buffer as usize + self.map.map_size as usize {
            self.cur += self.map.descriptor_size as usize;
            unsafe { Some(core::mem::transmute(p)) }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum EfiMemoryType {
    ReservedMemoryType,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    AcpiReclaimMemory,
    AcpiMemoryNvs,
    MemoryMappedIo,
    MemoryMappedIoPortSpace,
    PalCode,
    PersistentMemory,
    MaxMemoryType,
}

impl TryFrom<u32> for EfiMemoryType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < EfiMemoryType::MaxMemoryType as u32 {
            unsafe { Ok(core::mem::transmute(value)) }
        } else {
            Err(())
        }
    }
}

#[inline]
pub fn is_available(memory_type: EfiMemoryType) -> bool {
    memory_type == EfiMemoryType::BootServicesData ||
    memory_type == EfiMemoryType::BootServicesCode ||
    memory_type == EfiMemoryType::ConventionalMemory
}
