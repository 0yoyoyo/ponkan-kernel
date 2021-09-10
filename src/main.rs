#![no_std]
#![no_main]
#![feature(asm)]

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn kernel_main() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
