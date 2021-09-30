#![allow(dead_code)]

use crate::error::*;

use core::fmt;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

extern "C" {
    fn io_out32(addr: u16, data: u32);
    fn io_in32(addr: u16) -> u32;
}

fn write_address(address: u32) {
    unsafe {
        io_out32(CONFIG_ADDRESS, address);
    }
}

fn write_data(value: u32) {
    unsafe {
        io_out32(CONFIG_DATA, value);
    }
}

fn read_data() -> u32 {
    unsafe {
        io_in32(CONFIG_DATA)
    }
}

fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    let shl = |x: u32, bits: usize| {
        x << bits
    };

    shl(1, 31)
        | shl(bus as u32, 16)
        | shl(device as u32, 11)
        | shl(function as u32, 8)
        | (reg_addr as u32 & 0xfc)
}

pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    (read_data() & 0x0000ffff) as u16
}

pub fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    (read_data() >> 16) as u16
}

pub fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    ((read_data() >> 16) & 0x000000ff) as u8
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    write_address(make_address(bus, device, function, 0x08));
    let data = read_data();
    let base = ((data >> 24) & 0x000000ff) as u8;
    let sub = ((data >> 16) & 0x000000ff) as u8;
    let interface = ((data >> 8) & 0x000000ff) as u8;
    ClassCode { base, sub, interface }
}

pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

pub fn read_conf_reg(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    write_address(make_address(bus, device, function, reg_addr));
    read_data()
}

pub fn write_conf_reg(bus: u8, device: u8, function: u8, reg_addr: u8, value: u32) {
    write_address(make_address(bus, device, function, reg_addr));
    write_data(value);
}

pub fn is_single_function_device(header_type: u8) -> bool {
    header_type & 0x80 == 0x00
}

pub fn read_vendor_id_from_device(device: &Device) -> u16 {
    read_vendor_id(device.bus, device.device, device.function)
}

pub fn read_conf_reg_from_device(device: &Device, reg_addr: u8) -> u32 {
    read_conf_reg(device.bus, device.device, device.function, reg_addr)
}

pub fn write_conf_reg_from_device(device: &Device, reg_addr: u8, value: u32) {
    write_conf_reg(device.bus, device.device, device.function, reg_addr, value);
}

const fn calc_bar_address(bar_index: usize) -> u8 {
    0x10 + 4 * (bar_index as u8)
}

pub fn read_bar(device: &Device, bar_index: usize) -> Result<u64, PciError> {
    if bar_index >= 6 {
        return make_error!(PciErrorCode::IndexOutOfRange);
    }

    let addr = calc_bar_address(bar_index);
    let bar_lower = read_conf_reg(
        device.bus, device.device, device.function, addr) as u64;

    // 32 bit address
    if (bar_lower & 0x4) == 0x0 {
        return Ok(bar_lower);
    }

    // 64 bit address
    if bar_index >= 5 {
        return make_error!(PciErrorCode::IndexOutOfRange);
    }

    let bar_upper = read_conf_reg(
        device.bus, device.device, device.function, addr + 4) as u64;
    let bar = bar_lower | (bar_upper << 32);
    Ok(bar)
}

#[derive(Clone, Copy)]
pub struct ClassCode {
    base: u8,
    sub: u8,
    interface: u8,
}

impl fmt::LowerHex for ClassCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}XX",
            self.base, self.sub, self.interface)
    }
}

impl ClassCode {
    pub fn is_matched(&self, base: u8, sub: u8, interface: u8) -> bool {
        self.is_matched_base_and_sub(base, sub) &&
            self.interface == interface
    }

    pub fn is_matched_base_and_sub(&self, base: u8, sub: u8) -> bool {
        self.is_matched_base(base) && self.sub == sub
    }

    pub fn is_matched_base(&self, base: u8) -> bool {
        self.base == base
    }
}

#[derive(Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub class_code: ClassCode,
}

pub struct BusScanner {
    devices: [Device; 32],
    num_device: usize,
}

impl BusScanner {
    pub fn new() -> Self {
        let empty_class_code = ClassCode {
            base: 0,
            sub: 0,
            interface: 0,
        };
        let empty_device = Device {
            bus: 0,
            device: 0,
            function: 0,
            header_type: 0,
            class_code: empty_class_code,
        };
        Self {
            devices: [empty_device; 32],
            num_device: 0,
        }
    }

    pub fn devices(&self) -> &[Device] {
        &self.devices
    }

    pub fn num_device(&self) -> usize {
        self.num_device
    }

    pub fn scan_all_bus(&mut self) -> Result<(), PciError> {
        self.num_device = 0;

        let header_type = read_header_type(0, 0, 0);
        if is_single_function_device(header_type) {
            return self.scan_bus(0);
        }

        for function in 1..8 {
            if read_vendor_id(0, 0, function) == 0xffff {
                continue;
            }
            self.scan_bus(function)?;
        }

        Ok(())
    }

    fn scan_bus(&mut self, bus: u8) -> Result<(), PciError> {
        for device in 0..32 {
            if read_vendor_id(bus, device, 0) == 0xffff {
                continue;
            }
            self.scan_device(bus, device)?;
        }

        Ok(())
    }

    fn scan_device(&mut self, bus: u8, device: u8) -> Result<(), PciError> {
        self.scan_function(bus, device, 0)?;

        if is_single_function_device(read_header_type(bus, device, 0)) {
            return Ok(());
        }

        for function in 1..8 {
            if read_vendor_id(bus, device, function) == 0xffff {
                continue;
            }
            self.scan_function(bus, device, function)?;
        }

        Ok(())
    }

    fn scan_function(
        &mut self,
        bus: u8,
        device: u8,
        function: u8,
    ) -> Result<(), PciError> {
        let header_type = read_header_type(bus, device, function);
        let class_code = read_class_code(bus, device, function);

        self.add_device(bus, device, function, header_type, class_code)?;

        if class_code.is_matched_base_and_sub(0x06, 0x04) {
            let bus_numbers = read_bus_numbers(bus, device, function);
            let secondary_bus = ((bus_numbers >> 8) & 0x000000ff) as u8;
            return self.scan_bus(secondary_bus);
        }

        Ok(())
    }

    fn add_device(
        &mut self,
        bus: u8,
        device: u8,
        function: u8,
        header_type: u8,
        class_code: ClassCode,
    ) -> Result<(), PciError> {
        if self.num_device == self.devices.len() {
            return make_error!(PciErrorCode::Full);
        }

        self.devices[self.num_device] = Device {
            bus,
            device,
            function,
            header_type,
            class_code,
        };
        self.num_device += 1;

        Ok(())
    }
}
