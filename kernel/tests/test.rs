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



pub trait Testable {
    fn run(&self) -> ();
}

// in src/main.rs

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        println!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

#[cfg(test)]
mod tests {
    #[test_case]
    const TEST1: libkernel::UnitTest = libkernel::UnitTest {
            name: "test_runner_executes_in_kernel_mode",
            test_func: || {
                assert!(1 + 1 == 2);
            },
        };
}

// SPDX-License-Identifier: MIT OR Apache-2.0