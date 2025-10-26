#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]


extern crate alloc;

use alloc::boxed::Box;
use bootloader_api::{entry_point, BootInfo};


entry_point!(kernel_test_main, config = &rustyk::CONFIG);


fn kernel_test_main(boot_info: &'static mut BootInfo) -> ! {
    use rustyk::allocator;
    use rustyk::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    rustyk::kernel_init();

    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option().expect("No physical memory offset"));
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_regions)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    simple_allocation();
    rustyk::test::test_passed();
}


fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}
