#![no_std]
#![no_main]
use core::panic::PanicInfo;


use libkernel::{println, cpu};


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    libkernel::hlt_loop();
}