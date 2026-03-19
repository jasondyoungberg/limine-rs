# The Limine Boot Protocol For Rust
After the apparent abandonment of [limine-rs](https://github.com/jasondyoungberg/limine-rs) and by request of a personal friend, I have created an alternative set of Rust bindings for the Limine boot protocol.

## Supported Features
Base revision: 5

Requests:
 - Bootloader Info
 - Executable Command Line
 - Firmware Type
 - Stack Size
 - HHDM (Higher Half Direct Map)
 - Framebuffer
 - Paging Mode
 - MP (Multiprocessor)
 - RISC-V BSP Hart ID
 - Memory Map
 - Entry Point
 - Executable File
 - Module
 - RSDP
 - SMBIOS
 - EFI System Table
 - EFI Memory Map
 - Date at Boot
 - Executable Address
 - Device Tree Blob
 - Bootloader Performance
