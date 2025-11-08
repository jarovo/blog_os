// src/main.rs

use std::path::{Path, PathBuf};
mod qemu_virtual_machine;


fn main() -> Result<(), std::io::Error> {
    qemu_virtual_machine::boot_qemu( env!("BIOS_IMAGE_CARGO_BIN_FILE_RUSTYK_kernel"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::qemu_virtual_machine;


    #[test]
    fn heap_allocation() -> Result<(), std::io::Error> {
        qemu_virtual_machine::boot_qemu(
            env!("BIOS_IMAGE_CARGO_BIN_FILE_RUSTYK_test_heap_allocation"),
        )?;
        Ok(())
    }

    #[test]
    fn stack_overflow() -> Result<(), std::io::Error> {
        qemu_virtual_machine::boot_qemu(
            env!("BIOS_IMAGE_CARGO_BIN_FILE_RUSTYK_test_stack_overflow"),
        )?;
        Ok(())
    }
}