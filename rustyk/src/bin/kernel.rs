#![no_std]
#![no_main]


use rustyk::{kernel_main, CONFIG};

bootloader_api::entry_point!(kernel_main, config = &CONFIG);
