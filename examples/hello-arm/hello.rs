#![no_std]
#![no_main]

use core::arch::asm;

#[panic_handler]
fn _handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let s = "Hello, world!\n";

    asm! {
        // fd
        "mov x0, #1",
        // buf
        "mov x1, {0}",
        // count
        "mov x2, {1}",
        // write
        "mov x8, #0x40",
        "svc #0",
        in(reg) s.as_ptr(),
        in(reg) s.len(),
    }

    asm! {
        // status
        "mov x0, #1",
        // exit
        "mov x8, #0x5e",
        "svc #0",
    }

    core::hint::unreachable_unchecked()
}

