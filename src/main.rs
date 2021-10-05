#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]

mod panic;
mod graphics;
mod font;
mod frame_buffer_config;
mod write_buffer;
mod console;
mod pci;
mod asmfunc;
mod logger;
mod error;
mod usb;
mod mouse;

use graphics::{
    PixelColor, PixelWriter, Vector2D, Displacement,
    RGBResv8BitPerColorPixelWriter, BGRResv8BitPerColorPixelWriter,
    fill_rectangle, draw_rectangle,
};
use frame_buffer_config::{
    PixelFormat, FrameBufferConfig
};
pub use write_buffer::WriteBuffer;
use console::Console;
use pci::{
    BusScanner, Device, read_bar, read_class_code,
    read_conf_reg_from_device, write_conf_reg_from_device,
    read_vendor_id, read_vendor_id_from_device
};
use logger::*;
use usb::{
    XhciController, configure_port, process_event,
    set_default_mouse_observer,
};
use mouse::MouseCursor;

use core::{cell::RefCell, fmt::Write, mem::MaybeUninit};

static mut BUF_RGB: MaybeUninit<RGBResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_BGR: MaybeUninit<BGRResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_WRITER: MaybeUninit<RefCell<&mut dyn PixelWriter>> =
    MaybeUninit::uninit();
pub static mut BUF_CONSOLE: MaybeUninit<Console> = MaybeUninit::uninit();
static mut BUF_MOUSE: MaybeUninit<MouseCursor> = MaybeUninit::uninit();

macro_rules! _kprint {
    ($w:ident, $($arg:tt)*) => ({
        let mut buf = WriteBuffer::<1024>::new();
        $w!(buf, $($arg)*).unwrap();
        let console = unsafe {
            BUF_CONSOLE.assume_init_mut()
        };
        console.put_string(buf);
    });
}

#[allow(unused_macros)]
macro_rules! kprint {
    ($($arg:tt)*) => (_kprint!(write, $($arg)*));
}

#[allow(unused_macros)]
macro_rules! kprintln {
    () => (_kprint!(write, "\n"));
    ($($arg:tt)*) => (_kprint!(writeln, $($arg)*));
}

extern "C" fn mouse_observer(displacement_x: i8, displacement_y: i8) {
    let displacement = Displacement {
        x: displacement_x as isize,
        y: displacement_y as isize,
    };
    let mouse_cursor = unsafe {
        BUF_MOUSE.assume_init_mut()
    };
    mouse_cursor.move_relative(displacement);
}

impl BusScanner {
    fn switch_ehci_to_xhci(&self, xhc_device: &Device) {
        let mut intel_ehc_exist = false;
        for device in self.devices() {
            if device.class_code.is_matched(0x0c, 0x03, 0x20) &&
                read_vendor_id_from_device(device) == 0x8086 {
                intel_ehc_exist = true;
                break;
            }
        }
        if !intel_ehc_exist {
            return;
        }

        let superspeed_port = read_conf_reg_from_device(xhc_device, 0xdc);
        write_conf_reg_from_device(xhc_device, 0xd8, superspeed_port);
        let ehci_to_xhci_port = read_conf_reg_from_device(xhc_device, 0xd4);
        write_conf_reg_from_device(xhc_device, 0xd0, ehci_to_xhci_port);
        log!(Debug, "switch_ehci_to_xhci: SS = {:02x}, xHCI = {:02x}",
             superspeed_port, ehci_to_xhci_port);
    }
}

#[no_mangle]
pub extern "C" fn kernel_main(
    frame_buffer_config: &'static mut FrameBufferConfig,
) -> ! {
    let frame_width = frame_buffer_config.horisontal_resolution as usize;
    let frame_height = frame_buffer_config.vertical_resolution as usize;

    let pixel_writer: &mut dyn PixelWriter =
        match &frame_buffer_config.pixel_format
    {
        PixelFormat::kPixelRGBResv8BitPerColor => {
            unsafe {
                BUF_RGB.write(
                    RGBResv8BitPerColorPixelWriter(frame_buffer_config));
                BUF_RGB.assume_init_mut()
            }
        },
        PixelFormat::kPixelBGRResv8BitPerColor => {
            unsafe {
                BUF_BGR.write(
                    BGRResv8BitPerColorPixelWriter(frame_buffer_config));
                BUF_BGR.assume_init_mut()
            }
        },
    };
    let pixel_writer = unsafe {
        BUF_WRITER.write(RefCell::new(pixel_writer));
        BUF_WRITER.assume_init_ref()
    };

    let desktop_bg_color = PixelColor { r: 45, g: 115, b: 200 };
    let desktop_fg_color = PixelColor { r: 255, g: 255, b: 255 };
    fill_rectangle(
        pixel_writer,
        &Vector2D { x: 0, y: 0 },
        &Vector2D { x: frame_width, y: frame_height - 40 },
        &desktop_bg_color,
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D { x: 0, y: frame_height - 40 },
        &Vector2D { x: frame_width, y: 40 },
        &PixelColor { r: 10, g: 30, b: 50 },
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D { x: 0, y: frame_height - 40 },
        &Vector2D { x: frame_width / 4, y: 40 },
        &PixelColor { r: 120, g: 120, b: 120 },
    );
    draw_rectangle(
        pixel_writer,
        &Vector2D { x: 10, y: frame_height - 30 },
        &Vector2D { x: 20, y: 20 },
        &PixelColor { r: 50, g: 160, b: 50 },
    );

    unsafe {
        BUF_CONSOLE.write(
            Console::new(desktop_fg_color, desktop_bg_color, pixel_writer)
        );
    }
    kprintln!("Welcome to PonkanOS!");
    set_log_level(Warn);

    let initial_position = Vector2D {
        x: 300,
        y: 200,
    };
    unsafe {
        BUF_MOUSE.write(
            MouseCursor::new(pixel_writer, desktop_bg_color, initial_position)
        );
    }

    let mut scanner = BusScanner::new();
    match scanner.scan_all_bus() {
        Ok(_) => {
            log!(Debug, "scan_all_bus: Success");
        },
        Err(err) => {
            log!(Debug, "scan_all_bus: Error ({:?})", err.code);
        },
    }

    for device in scanner.devices().iter().take(scanner.num_device()) {
        let vendor_id = read_vendor_id(
            device.bus, device.device, device.function);
        let class_code = read_class_code(
            device.bus, device.device, device.function);
        log!(Debug, "{}.{}.{}: vendor {:04x}, class {:08x}, header {:02x}",
            device.bus, device.device, device.function,
            vendor_id, class_code, device.header_type);
    }

    let mut xhc_device = None;
    for device in scanner.devices().iter().take(scanner.num_device()) {
        if device.class_code.is_matched(0x0c, 0x03, 0x30) {
            xhc_device = Some(device);

            if read_vendor_id_from_device(device) == 0x8086 {
                break;
            }
        }
    }

    if let Some(device) = xhc_device {
        log!(Info, "xHC has been found: {}.{}.{}",
            device.bus, device.device, device.function);

        match read_bar(device, 0) {
            Ok(bar) => {
                let xhc_mmio_base = bar & !0xf;
                log!(Debug, "xHC mmio_base = {:08x}", xhc_mmio_base);

                let mut xhc = XhciController::new(xhc_mmio_base);

                if read_vendor_id_from_device(device) == 0x8086 {
                    scanner.switch_ehci_to_xhci(device);
                }

                let err_code = xhc.initialize();
                log!(Debug, "xhc.initialize: {}", err_code);

                log!(Info, "xHC starting");
                xhc.run();

                set_default_mouse_observer(mouse_observer);

                for i in 0..xhc.max_ports() {
                    let mut port = xhc.port_at(i);
                    log!(Debug, "Port {}: is_connected={}",
                         i, port.is_connected());

                    if port.is_connected() &&
                        configure_port(&mut xhc, &mut port) != 0 {
                        log!(Error, "Failed to configure port");
                        continue;
                    }
                }

                loop {
                    if process_event(&mut xhc) != 0 {
                        log!(Error, "Error while process_event");
                    }
                }
            },
            Err(err) => {
                log!(Debug, "read_bar: Error ({:?})", err.code);
            },
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
