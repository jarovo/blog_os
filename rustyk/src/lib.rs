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
use x86_64::{VirtAddr, structures::paging::{Translate, Page, OffsetPageTable}};
use alloc::sync::Arc;
use core::fmt::Write;
use spin::Mutex;
use task::{Task, executor::Executor};

use crate::{keyboard::print_keypresses, clock::Ticker};

pub mod test;
pub mod task;
pub mod clock;
pub mod keyboard;

pub fn kernel_init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {

    kernel_init();
    let mut console = console::Console::new();

    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option()
        .expect("Physical memory offset is required but not provided by the bootloader."));
    
    println!("Physical memory offset: {:#016x}", phys_mem_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization is required but failed.");

    writeln!(console, "Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch()).unwrap();

    // Display memory regions.
    for region in boot_info.memory_regions.iter() {
        writeln!(console, "Memory region: {:#016x} - {:#016x} ({:?})",
                 region.start,
                 region.end,
                 region.kind).unwrap();
    }

    print_mappings(&mut console, boot_info, &mut mapper);

    // Map an unused page.
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let fb_info = boot_info.framebuffer.take().expect("Expected to find a framebuffer.");
    console.add_screen(console::Screen::BootloaderScreen(console::BootloaderScreen::new(fb_info)));
    //console.add_screen(console::Screen::VGAScreen1280x800x256(console::VGAScreen1280x800x256::new()));
    console.clear_screen().expect("Expected to be able to clear the screen.");

    let shared_console = Arc::new(Mutex::new(console));

    let mut executor = Executor::new();
    executor.spawn(Task::new(task_42_caller(shared_console.clone())));
    executor.spawn(Task::new(print_keypresses(shared_console.clone())));
    executor.spawn(Task::new(console_printing_task(shared_console.clone(), 1)));
    executor.spawn(Task::new(console_printing_task(shared_console.clone(), 2)));
    executor.run(); 

    #[cfg(feature = "with-self-tests")]
    {
        println!("In test mode!");
        run_tests();
        println!("It did not crash!");
        qemu_exit_success();
    }    
}

async fn task_42(console: Arc<Mutex<console::Console>>) -> u32 {
    writeln!(console.lock(), "Task 42 is running!").unwrap();
    42
}

async fn task_42_caller(console: Arc<Mutex<console::Console>>) {
    let result = task_42(console.clone()).await;
    writeln!(console.lock(), "Task 42 returned: {}", result).unwrap();
}

async fn console_printing_task(console: Arc<Mutex<console::Console>>, task_id: u64) {
    let mut count = 0;
    let mut ticker = Ticker::new(100); // Tick every 100 clock ticks
    loop {
        {
            count += 1;
            writeln!(console.lock(), "Task {} is running! Count: {}. Clock ticks: {}", task_id, count, clock::Clock.ticks()).unwrap();
            ticker.tick().await;
        }
    }
}

fn print_mappings(console: &mut console::Console,
                           boot_info: &bootloader_api::BootInfo,
                           mapper: &OffsetPageTable)
{
    for address in boot_info.memory_regions.iter().map(|r| r.start) {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        writeln!(console, "{:?} -> {:?}", virt, phys).unwrap();
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

#[cfg(feature = "with-self-tests")]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}


#[cfg(feature = "with-self-tests")]
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

#[cfg(feature = "with-self-tests")]
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