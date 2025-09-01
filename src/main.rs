#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

use core::panic::PanicInfo;

mod serial;
mod vga_buffer;
mod limine_vga;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Panic occurred: {}", _info).unwrap();
    loop {}
}

fn main() -> ! {
   use core::fmt::Write;
   let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Hello World from serial!").unwrap();
    println!("Hello World from VGA!");
    let mut lwriter = limine_vga::Writer::new();
    lwriter.write("Hello World from limine VGA!");
    lwriter.write("Hello World from limine VGA! Second line!");
    loop {}
    
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    main()
}