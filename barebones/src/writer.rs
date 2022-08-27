use core::fmt;
use core::fmt::Write;

use limine::LimineTerminalResponse;

// Used to write to the screen.
use crate::TERMINAL_REQUEST;

struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        static mut CACHED: Option<&'static LimineTerminalResponse> = None;

        unsafe {
            if let Some(writer) = CACHED {
                let terminal = &writer.terminals()[0];
                writer.write().unwrap()(&terminal, s);
            } else {
                let response = TERMINAL_REQUEST.get_response().get().unwrap();
                let terminal = &response.terminals()[0];
                let writer = response.write().unwrap();

                writer(&terminal, s);

                // initialize the cached response
                CACHED = Some(response);
            }
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::writer::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut writer = Writer;
    // UNWRAP: We always return `Ok(())` inside `write_str` so this is unreachable.
    writer.write_fmt(args).unwrap();
}
