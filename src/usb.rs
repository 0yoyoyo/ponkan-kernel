type FFIPointer = u64;
type FFIErrorCode = u32;

#[link(name="driver")]
extern "C" {
    fn XHCI_Controller_New(mmio_base: u64) -> FFIPointer;
    fn XHCI_Controller_Delete(controller: FFIPointer);
    fn XHCI_Controller_Initialize(controller: FFIPointer) -> FFIErrorCode;
    fn XHCI_Controller_Run(controller: FFIPointer) -> FFIErrorCode;
    fn XHCI_Controller_PrimaryEventRing(controller: FFIPointer)-> FFIPointer;
    fn XHCI_Controller_PortAt(
        controller: FFIPointer,
        port_num: u8
    )-> FFIPointer;
    fn XHCI_Controller_MaxPorts(controller: FFIPointer) -> u8;
    fn XHCI_Port_IsConnected(port: FFIPointer) -> bool;
    fn XHCI_EventRing_HasFront(event_ring: FFIPointer) -> bool;
    fn XHCI_ConfigurePort(
        controller: FFIPointer,
        port: FFIPointer
    ) -> FFIErrorCode;
    fn XHCI_ProcessEvent(controller: FFIPointer) -> FFIErrorCode;
    fn XHCI_Mouse_SetDefaultObserver(
        observer: extern "C" fn(i8, i8),
    );
}

pub struct XhciController(FFIPointer);
pub struct XhciEventRing(FFIPointer);
pub struct XhciPort(FFIPointer);

impl XhciController {
    pub fn new(mmio_base: u64) -> Self {
        unsafe {
            XhciController(XHCI_Controller_New(mmio_base))
        }
    }

    pub fn initialize(&mut self) -> FFIErrorCode {
        unsafe {
            XHCI_Controller_Initialize(self.0)
        }
    }

    pub fn run(&mut self) -> FFIErrorCode {
        unsafe {
            XHCI_Controller_Run(self.0)
        }
    }

    pub fn primary_event_ring(&mut self) -> XhciEventRing {
        unsafe {
            XhciEventRing(XHCI_Controller_PrimaryEventRing(self.0))
        }
    }

    pub fn port_at(&mut self, port_num: u8) -> XhciPort {
        unsafe {
            XhciPort(XHCI_Controller_PortAt(self.0, port_num))
        }
    }

    pub fn max_ports(&mut self) -> u8 {
        unsafe {
            XHCI_Controller_MaxPorts(self.0)
        }
    }
}

impl Drop for XhciController {
    fn drop(&mut self) {
        unsafe {
            XHCI_Controller_Delete(self.0)
        }
    }
}

impl XhciEventRing {
    pub fn has_front(&mut self) -> bool {
        unsafe {
            XHCI_EventRing_HasFront(self.0)
        }
    }
}

impl XhciPort {
    pub fn is_connected(&mut self) -> bool {
        unsafe {
            XHCI_Port_IsConnected(self.0)
        }
    }
}

pub fn configure_port(
    controller: &mut XhciController,
    port: &mut XhciPort
) -> FFIErrorCode {
    unsafe {
        XHCI_ConfigurePort(controller.0, port.0)
    }
}

pub fn process_event(
    controller: &mut XhciController,
) -> FFIErrorCode {
    unsafe {
        XHCI_ProcessEvent(controller.0)
    }
}

pub fn set_default_mouse_observer(
    observer: extern "C" fn(i8, i8),
) {
    unsafe {
        XHCI_Mouse_SetDefaultObserver(observer);
    }
}
