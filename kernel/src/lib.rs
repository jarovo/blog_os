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
pub mod gdt;

fn kernel_init(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    gdt::init();
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
    
    println!("It did not crash!");
    hlt_loop();
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

#[cfg(feature = "with-tests")]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}


#[cfg(feature = "with-tests")]
fn test_trivial() {
    assert_eq!(1, 1);
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
        &test_trivial,
        // Add more test functions here as needed
    ];
    test_runner(tests);
}



pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
