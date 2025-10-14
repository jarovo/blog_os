#![feature(custom_test_frameworks)]
#![test_runner(libkernel::test_runner)]
#![feature(format_args_nl)]
#![no_main]
#![no_std]

use libkernel::println;

unsafe fn kernel_init() -> ! {

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