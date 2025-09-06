#![no_std]
#![no_main]

use libkernel::{println, cpu};
use bootloader_api::{entry_point, BootInfo};

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};

entry_point!(kernel_init, config = &CONFIG);


fn kernel_main() -> ! {
    // Initialize the kernel

    cpu::qemu_exit_success();
}

pub fn kernel_init(boot_info: &'static mut BootInfo) -> ! {
    // Initialize the console

    println!("Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch());
    println!("All writes written now!");

    kernel_main()
}