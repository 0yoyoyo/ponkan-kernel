#![no_std]
#![no_main]
#![feature(asm)]

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn kernel_main(
    frame_buffer_base: u64,
    frame_buffer_size: u64,
) -> ! {
    let frame_buffer_base =
        core::mem::transmute::<_, *mut u8>(frame_buffer_base);
    let frame_buffer_size = frame_buffer_size as usize;
    let frame_buffer =
        core::slice::from_raw_parts_mut(frame_buffer_base, frame_buffer_size);
    for (i, frame) in frame_buffer.iter_mut().enumerate() {
        *frame = (i % 256) as u8;
    }

    loop {
        asm!("hlt");
    }
}
