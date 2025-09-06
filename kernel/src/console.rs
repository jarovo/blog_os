/* The console of the kernel writing to serial port and framebuffer. */

use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::serial;


lazy_static! {
    pub static ref WRITER: Mutex<serial::Writer> = Mutex::new(serial::Writer::new(0x3F8)); // COM1
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}