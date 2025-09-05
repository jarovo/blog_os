#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::fmt::Write;
use core::panic::PanicInfo;

mod serial;
mod limine_vga;

use linked_list_allocator::LockedHeap;
use bootloader_api::{entry_point, BootInfo};

// 1. Define heap size
const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

// 2. Allocate static buffer
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

// 3. Install allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();



#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialize the kernel

    /*unsafe {
        ALLOCATOR.lock().init(core::ptr::addr_of_mut!(HEAP).cast(), HEAP_SIZE);
    }*/

    writes();
    println!("Boot info API version: {}.{}.{}", boot_info.api_version.version_major(), boot_info.api_version.version_minor(), boot_info.api_version.version_patch());
    println!("All writes written now!");
    
    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Panic occurred: {}", _info).unwrap();
    loop {}
}

fn writes() {
    let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Hello World from serial!").unwrap();
    println!("Hello World from VGA!");
    for i in 0..10 {
        println!("Line number {}", i);
    }
}