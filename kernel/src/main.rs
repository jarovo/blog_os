#![no_std]
#![no_main]


use libkernel::{kernel_init, CONFIG};

bootloader_api::entry_point!(kernel_init, config = &CONFIG);
