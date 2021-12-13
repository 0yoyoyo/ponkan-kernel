#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]

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
mod interrupt;
mod queue;
mod memory_map;
mod segment;
mod x86_descriptor;
mod paging;
mod memory_manager;
mod alloc_support;

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
    BusScanner, Device, MsiDeliveryMode, MsiTriggerMode,
    read_bar, read_class_code, read_vendor_id, read_conf_reg_from_device,
    write_conf_reg_from_device, read_vendor_id_from_device,
    configure_msi_fixed_destination,
};
use logger::*;
use usb::{
    XhciController, configure_port, process_event,
    set_default_mouse_observer,
};
use mouse::MouseCursor;
use interrupt::{
    InterruptVector, ExceptionStackFrame,
    notify_end_of_interrupt, get_cs, load_idt, make_id_attr, set_idt_entry,
    IDT,
};
use queue::ArrayQueue;
use memory_map::{MemoryMap, is_available, UEFI_PAGE_SIZE};
use x86_descriptor::GateDescriptorType;
use segment::setup_segments;
use paging::setup_identity_page_table;
use memory_manager::{BitmapMemoryManager, FrameId, BYTE_PER_FRAME};

use core::{
    cell::RefCell, convert::TryInto, fmt::Write, mem::size_of_val,
    mem::MaybeUninit, ptr::read_volatile
};

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

extern "C" {
    fn set_ds_all(value: u16);
    fn set_cs_ss(cs: u16, ss: u16);
}

#[derive(Clone, Copy, Debug)]
enum MassageType {
    InterruptXhci,
}

#[derive(Clone, Copy)]
struct Message {
    m_type: Option<MassageType>,
}

static mut MAIN_QUEUE_DATA: [Message; 32] =[Message { m_type: None }; 32];

static mut BUF_RGB: MaybeUninit<RGBResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_BGR: MaybeUninit<BGRResv8BitPerColorPixelWriter> =
    MaybeUninit::uninit();
static mut BUF_WRITER: MaybeUninit<RefCell<&mut dyn PixelWriter>> =
    MaybeUninit::uninit();
pub static mut BUF_CONSOLE: MaybeUninit<Console> = MaybeUninit::uninit();
static mut BUF_MOUSE: MaybeUninit<MouseCursor> = MaybeUninit::uninit();
static mut BUF_XHC: MaybeUninit<XhciController> = MaybeUninit::uninit();
static mut BUF_QUEUE: MaybeUninit<ArrayQueue<Message, 32>> =
    MaybeUninit::uninit();
static mut BUF_FBCONFIG: MaybeUninit<FrameBufferConfig> = MaybeUninit::uninit();
static mut BUF_MEMMAP: MaybeUninit<MemoryMap> = MaybeUninit::uninit();
pub static mut BUF_MEMMNG: MaybeUninit<BitmapMemoryManager> =
    MaybeUninit::uninit();

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

extern "x86-interrupt" fn interrupt_handler_xhci(
    _stack_frame: ExceptionStackFrame,
) {
    let main_queue = unsafe {
        BUF_QUEUE.assume_init_mut()
    };
    main_queue
        .push(Message { m_type: Some(MassageType::InterruptXhci) })
        .unwrap();
    unsafe {
        notify_end_of_interrupt();
    }
}

#[repr(align(16))]
pub struct KernelMainStack([u8; 1024 * 1024]);

#[no_mangle]
pub static mut KERNEL_MAIN_STACK: KernelMainStack =
    KernelMainStack([0; 1024 * 1024]);

#[no_mangle]
pub extern "C" fn kernel_main_new_stack(
    frame_buffer_config_ref: &'static mut FrameBufferConfig,
    memory_map_ref: &'static MemoryMap,
) -> ! {
    let frame_buffer_config = unsafe {
        BUF_FBCONFIG.write(*frame_buffer_config_ref)
    };
    let memory_map = unsafe {
        BUF_MEMMAP.write(*memory_map_ref)
    };

    let frame_width = frame_buffer_config.horisontal_resolution as usize;
    let frame_height = frame_buffer_config.vertical_resolution as usize;

    let pixel_writer: &mut dyn PixelWriter =
        match &frame_buffer_config.pixel_format
    {
        PixelFormat::kPixelRGBResv8BitPerColor => {
            unsafe {
                BUF_RGB.write(RGBResv8BitPerColorPixelWriter(frame_buffer_config))
            }
        },
        PixelFormat::kPixelBGRResv8BitPerColor => {
            unsafe {
                BUF_BGR.write(BGRResv8BitPerColorPixelWriter(frame_buffer_config))
            }
        },
    };
    let pixel_writer = unsafe {
        BUF_WRITER.write(RefCell::new(pixel_writer))
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

    setup_segments();

    unsafe {
        let kernel_cs = 1 << 3;
        let kernel_ss = 2 << 3;
        set_ds_all(0);
        set_cs_ss(kernel_cs, kernel_ss);
    }

    setup_identity_page_table();

    let memory_manager = unsafe {
        BUF_MEMMNG.write(BitmapMemoryManager::new())
    };
    let mut available_end = 0;
    for desc in memory_map.iter() {
        if available_end < desc.physical_start {
            memory_manager.mark_allocated(
                FrameId::new(available_end / BYTE_PER_FRAME),
                (desc.physical_start - available_end) / BYTE_PER_FRAME,
            );
        }

        let physical_end =
            desc.physical_start + desc.number_of_pages as usize * UEFI_PAGE_SIZE;
        if is_available(desc.memory_type.try_into().unwrap()) {
            available_end = physical_end;
        } else {
            memory_manager.mark_allocated(
                FrameId::new(desc.physical_start / BYTE_PER_FRAME),
                desc.number_of_pages as usize * UEFI_PAGE_SIZE / BYTE_PER_FRAME,
            );
        }
    }

    memory_manager.set_memory_range(
        FrameId::new(1),
        FrameId::new(available_end / BYTE_PER_FRAME),
    );

    let initial_position = Vector2D {
        x: 300,
        y: 200,
    };
    unsafe {
        BUF_MOUSE.write(
            MouseCursor::new(pixel_writer, desktop_bg_color, initial_position)
        );
    }

    unsafe {
        BUF_QUEUE.write(ArrayQueue::new(&mut MAIN_QUEUE_DATA));
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

        unsafe {
            let cs = get_cs();
            let attr = make_id_attr(GateDescriptorType::InterruptGate, 0);
            set_idt_entry(
                &mut IDT[InterruptVector::Xhci as usize],
                attr,
                interrupt_handler_xhci as usize as u64,
                cs,
            );
            load_idt((size_of_val(&IDT) - 1) as u16, &IDT as *const _ as u64);

            let bsp_local_apic_id =
                (read_volatile(0xfee00020 as *const u32) >> 24) & 0x000000ff;
            configure_msi_fixed_destination(
                device,
                bsp_local_apic_id,
                MsiTriggerMode::Level,
                MsiDeliveryMode::Fixed,
                InterruptVector::Xhci as u32,
                0,
            ).unwrap();
        }

        match read_bar(device, 0) {
            Ok(bar) => {
                let xhc_mmio_base = bar & !0xf;
                log!(Debug, "xHC mmio_base = {:08x}", xhc_mmio_base);

                let xhc = unsafe {
                    BUF_XHC.write(XhciController::new(xhc_mmio_base))
                };

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
                        configure_port(xhc, &mut port) != 0 {
                        log!(Error, "Failed to configure port");
                        continue;
                    }
                }

                let main_queue = unsafe {
                    BUF_QUEUE.assume_init_mut()
                };
                loop {
                    unsafe {
                        asm!("cli");
                    }
                    if main_queue.count() == 0 {
                        unsafe {
                            asm!("sti", "hlt");
                        }
                        continue;
                    }

                    let message = main_queue.pop().unwrap();
                    unsafe {
                        asm!("sti");
                    }

                    #[allow(unreachable_patterns)]
                    match message.m_type.unwrap() {
                        MassageType::InterruptXhci => {
                            let xhc = unsafe {
                                BUF_XHC.assume_init_mut()
                            };
                            while xhc.primary_event_ring().has_front() {
                                if process_event(xhc) != 0 {
                                    log!(Error, "Error while process_event");
                                }
                            }
                        },
                        _ => log!(
                            Error,
                            "Unknown message type: {:?}",
                            message.m_type,
                        ),
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
