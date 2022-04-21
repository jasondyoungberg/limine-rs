#![no_std]
#![no_main]

mod writer;

use core::panic::PanicInfo;
use limine_rs::*;

static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// define the kernel's entry point function
#[no_mangle]
extern "C" fn x86_64_barebones_main() -> ! {
    println!("Hello, rusty world!\n");

    let bootloader_info = BOOTLOADER_INFO.response.get().unwrap();

    println!(
        "bootloader: (name={}, version={})",
        bootloader_info.name.to_string(),
        bootloader_info.version.to_string()
    );

    loop {}
}
