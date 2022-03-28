#![no_std]
#![no_main]

use core::panic::PanicInfo;
use limine_rs::*;

#[link_section = ".limine_requests"]
#[used]
static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// define the kernel's entry point function
#[no_mangle]
extern "C" fn x86_64_barebones_main() -> ! {
    let terminal_response = TERMINAL_REQUEST.response.get().unwrap();
    let term_write = terminal_response.write();

    term_write("Hello, rusty world!");

    loop {}
}
