use crate::cpu;
use super::println;


pub fn test_passed() -> ! {
    println!("Test passed. OK1234");
    cpu::qemu_exit_success();
}

pub fn test_failed() -> ! {
    panic!("Test failed");
}