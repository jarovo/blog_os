/* Inspired by https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/12_integrated_testing/README.md */

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

pub mod console;
mod serial;
pub mod cpu;

/// Unit test container.
pub struct UnitTest {
    /// Name of the test.
    pub name: &'static str,

    /// Function pointer to the test.
    pub test_func: fn(),
}

/// The default runner for unit tests.
pub fn test_runner(tests: &[&UnitTest]) {
    // This line will be printed as the test header.
    println!("Running {} tests", tests.len());

    for (i, test) in tests.iter().enumerate() {
        print!("{:>3}. {:.<58}", i + 1, test.name);

        // Run the actual test.
        (test.test_func)();

        // Failed tests call panic!(). Execution reaches here only if the test has passed.
        println!("[ok]")
    }
}


fn kernel_init(boot_info: &'static mut bootloader_api::BootInfo) -> ! {

    println!("Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch());

    #[cfg(test)]
    println!("All writes written now!");

    cpu::qemu_exit_success();
}


const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};

bootloader_api::entry_point!(kernel_init, config = &CONFIG);


#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! { 
    println!("[failed]\n");
    println!("Error: {}\n", info);
    cpu::qemu_exit_failure();
}