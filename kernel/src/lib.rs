/* Inspired by https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/12_integrated_testing/README.md */

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(test_runner)]

pub mod console;
mod serial;
pub mod cpu;


fn kernel_init(boot_info: &'static mut bootloader_api::BootInfo) -> ! {

    println!("Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch());

    #[cfg(test)]
    {
        println!("In test!");
        //test_main();
    }
    println!("All writes written now!");

    cpu::qemu_exit_success();
}

/// Entry point for `cargo test`
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_init, config = &CONFIG);




// use hermit::{print, println};
pub trait Testable {
	fn run(&self) -> ();
}

impl<T> Testable for T
where
	T: Fn(),
{
	fn run(&self) {
		print!("{}...\t", core::any::type_name::<T>());
		self();
		println!("[ok]");
	}
}

pub fn test_runner(tests: &[&dyn Testable]) {
	println!("Running {} tests", tests.len());
	for test in tests {
		test.run();
	}
}

#[test_case]
fn add_one() {
	let x = 1 + 2;
	assert_eq!(x, 3);
}


#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! { 
    println!("[failed]\n");
    println!("Error: {}\n", info);
    cpu::qemu_exit_failure();
}