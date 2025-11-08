#![feature(custom_test_frameworks)]
#![test_runner(libkernel::test_runner)]
#![feature(format_args_nl)]
#![no_main]
#![no_std]

use libkernel::println;

bootloader_api::entry_point!(kernel_test_init, config = &libkernel::CONFIG);

pub fn kernel_test_init(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {

    println!("OK1234");

    libkernel::cpu::qemu_exit_success();
}



#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}