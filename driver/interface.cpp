#include "usb/xhci/xhci.hpp"
#include "usb/classdriver/mouse.hpp"

using namespace usb;

char controller_buf[sizeof(xhci::Controller)];
char port_buf[sizeof(xhci::Port)];

void _XHCI_Mouse_SetDefaultObserver(
    std::function<HIDMouseDriver::ObserverType> observer
) {
    HIDMouseDriver::default_observer = observer;
    return;
}

extern "C" {
    xhci::Controller *XHCI_Controller_New(uintptr_t mmio_base) {
        return new(controller_buf) xhci::Controller{mmio_base};
    }

    void XHCI_Controller_Delete(xhci::Controller *controller) {
        return;
    }

    uint32_t XHCI_Controller_Initialize(xhci::Controller *controller) {
        Error err = controller->Initialize();
        return (uint32_t)err.Cause();
    }

    uint32_t XHCI_Controller_Run(xhci::Controller *controller) {
        Error err = controller->Run();
        return (uint32_t)err.Cause();
    }

    xhci::EventRing *XHCI_Controller_PrimaryEventRing(
        xhci::Controller *controller
    ) {
        return controller->PrimaryEventRing();
    }

    xhci::Port *XHCI_Controller_PortAt(
        xhci::Controller *controller,
        uint8_t port_num
    ) {
        return new(port_buf) xhci::Port (controller->PortAt(port_num));
    }

    uint8_t XHCI_Controller_MaxPorts(xhci::Controller *controller) {
        return controller->MaxPorts();
    }

    bool XHCI_EventRing_HasFront(xhci::EventRing *event_ring) {
        return event_ring->HasFront();
    }

    bool XHCI_Port_IsConnected(xhci::Port *port) {
        return port->IsConnected();
    }

    uint32_t XHCI_ConfigurePort(
        xhci::Controller *controller,
        xhci::Port *port
    ) {
        Error err = xhci::ConfigurePort(*controller, *port);
        return (uint32_t)err.Cause();
    }

    uint32_t XHCI_ProcessEvent(xhci::Controller *controller) {
        Error err = xhci::ProcessEvent(*controller);
        return (uint32_t)err.Cause();
    }

    void XHCI_Mouse_SetDefaultObserver(
        void (*observer)(int8_t, int8_t)
    ) {
        _XHCI_Mouse_SetDefaultObserver(observer);
        return;
    }
}
