/* Inspired by https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/12_integrated_testing/README.md */

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(test_runner)]
#![feature(abi_x86_interrupt)]

pub mod console;
mod serial;
pub mod cpu;
pub mod interrupts;

fn kernel_init(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    interrupts::init_idt();

    println!("Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch());

    #[cfg(feature = "with-tests")]
    {
        println!("In test mode!");
        run_tests();
    }
    
    cpu::qemu_exit_success();
}

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};


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
		print!("{}... ", core::any::type_name::<T>());
		self();
		println!("[ok]");
	}
}

fn add_one() {
	let x = 1 + 2;
	assert_eq!(x, 3);
}

fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

pub fn test_runner(tests: &[&dyn Testable]) {
	println!("Running {} tests", tests.len());
	for test in tests {
		test.run();
	}
    println!("[test did not panic]");
    }

#[cfg(feature = "with-tests")]
pub fn run_tests() {
    let tests: &[&dyn Testable] = &[
        &test_breakpoint_exception,
        &add_one,
        // Add more test functions here as needed
    ];
    test_runner(tests);
}


#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! { 
    println!("[failed]\n");
    println!("Error: {}\n", info);
    cpu::qemu_exit_failure();
}