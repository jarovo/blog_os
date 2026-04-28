use crate::cpu::qemu_exit_failure;
use crate::println;

use nostd::panic::PanicInfo;


#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    qemu_exit_failure();
}