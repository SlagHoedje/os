use core::fmt;
use core::fmt::Write;

/// Print something to the VGA Buffer. Calls `driver::vga::_print internally`. Line breaks will not
/// be automatically added.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::macros::_print(format_args!($($arg)*)));
}

/// Prints something to the VGA Buffer. A line break is automatically appended. An ANSI escape code
/// to reset the colors is also appended.
#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\x1b[37m\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\x1b[37m\n", format_args!($($arg)*)));
}

/// Internal function used by the `kprint!` macro.
pub fn _print(args: fmt::Arguments) {
    crate::driver::vga::WRITER.lock().write_fmt(args).unwrap();
    crate::driver::uart16550::UART.lock().write_fmt(args).unwrap();
}