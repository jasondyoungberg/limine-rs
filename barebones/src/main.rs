#![no_std]
#![no_main]

mod writer;

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
    println!("Hello, rusty world!");

    loop {}
}
