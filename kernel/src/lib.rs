/* Inspired by https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/12_integrated_testing/README.md */
#![no_main]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(test_runner)]
#![feature(abi_x86_interrupt)]

extern crate alloc; 

pub mod allocator;
pub mod memory;
pub mod console;
mod serial;
pub mod cpu;
mod interrupts;
pub mod gdt;
pub mod panicking;
use x86_64::{VirtAddr, structures::paging::Translate, structures::paging::Page, structures::paging::OffsetPageTable};
use core::fmt::Write;
pub mod test;

use crate::cpu::qemu_exit_success;


pub fn kernel_init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {

    kernel_init();

    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option().expect("No physical memory offset"));
    println!("Physical memory offset: {:#016x}", phys_mem_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed.");

    println!("Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch());

    // Display memory regions.
    for region in boot_info.memory_regions.iter() {
        println!("Memory region: {:#016x} - {:#016x} ({:?})",
                 region.start,
                 region.end,
                 region.kind);
    }

    print_mappings(boot_info, &mut mapper);

    // Map an unused page.
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let mut console = console::Console::new_from_bootinfo(
        boot_info.framebuffer.as_mut().expect("Failed to create console: No framebuffer found"));

    for i in 0..10 {
        writeln!(console, "Hello World! {}", i).ok();
    }
 

    #[cfg(feature = "with-tests")]
    {
        println!("In test mode!");
        run_tests();
    }
    
    println!("It did not crash!");
    qemu_exit_success();
}


fn print_mappings(boot_info: &bootloader_api::BootInfo, mapper: &OffsetPageTable) {
        for address in boot_info.memory_regions.iter().map(|r| r.start) {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
}

pub const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

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