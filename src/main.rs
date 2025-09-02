#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate alloc;

use core::panic::PanicInfo;

mod serial;
mod limine_vga;

use linked_list_allocator::LockedHeap;

// 1. Define heap size
const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

// 2. Allocate static buffer
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

// 3. Install allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();


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
    println!("Hello World from limine VGA! Second line!");
    loop {}
    
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        ALLOCATOR.lock().init(core::ptr::addr_of_mut!(HEAP).cast(), HEAP_SIZE);
    }

    main()
}
