use core::fmt;
use core::fmt::Write;

// Used to write to the screen.
use crate::TERMINAL_REQUEST;

struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // NOTE: if using this writer as a template for your own, you might want to cache
        // the result of the response by wrapping this statement in a once cell for performence.
        let writer = TERMINAL_REQUEST.response.get().unwrap().write();
        writer(s);

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
