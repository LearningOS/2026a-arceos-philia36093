#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// The guest "kernel" runs in VS-mode. It passes two magic values in `a0`/`a1`
/// and then invokes the legacy SBI shutdown call (`a7 = 8`), which traps to the
/// hypervisor so that it can shut the VM down.
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "li a0, 0x6688",
        "li a1, 0x1234",
        "li a7, 8",
        "ecall",
        options(noreturn)
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
