#![no_std]
#![no_main]


use libkernel::{kernel_main, CONFIG};

bootloader_api::entry_point!(kernel_main, config = &CONFIG);
