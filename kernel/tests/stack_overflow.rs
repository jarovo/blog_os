#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use libkernel::{print, println, cpu};
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use core::panic::PanicInfo;


bootloader_api::entry_point!(kernel_test_init, config = &libkernel::CONFIG);


pub fn kernel_test_init(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    println!("Boot info API version: {}.{}.{}",
             boot_info.api_version.version_major(),
             boot_info.api_version.version_minor(),
             boot_info.api_version.version_patch());
    print!("stack_overflow::stack_overflow...\t");
    libkernel::gdt::init();
    init_test_idt();
    stack_overflow();
    panic!("Execution continued after stack overflow");
}


#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

lazy_static! {  
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(libkernel::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}


pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("[ok]");
    cpu::qemu_exit_success();
}