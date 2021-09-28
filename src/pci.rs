#![allow(dead_code)]

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

pub fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x08));
    read_data()
}

pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

pub fn is_single_function_device(header_type: u8) -> bool {
    header_type & 0x80 == 0x00
}

#[derive(Debug)]
pub enum PciError {
    Full,
    Empty,
}

#[derive(Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
}

pub struct BusScanner {
    devices: [Device; 32],
    num_device: usize,
}

impl BusScanner {
    pub fn new() -> Self {
        let empty_device = Device {
            bus: 0,
            device: 0,
            function: 0,
            header_type: 0,
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
        self.add_device(bus, device, function, header_type)?;

        let class_code = read_class_code(bus, device, function);
        let base = ((class_code >> 24) & 0x000000ff) as u8;
        let sub = ((class_code >> 16) & 0x000000ff) as u8;

        if base == 0x06 && sub == 0x04 {
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
    ) -> Result<(), PciError> {
        if self.num_device == self.devices.len() {
            return Err(PciError::Full);
        }

        self.devices[self.num_device] = Device {
            bus,
            device,
            function,
            header_type,
        };
        self.num_device += 1;

        Ok(())
    }
}
