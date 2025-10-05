
use std::path::PathBuf;

fn main() {
    // Re-run if the selected kernel path changes
    println!("cargo:rerun-if-env-changed=CUSTOM_KERNEL_PATH");

    // set by cargo, build scripts should use this directory for output files
    let _out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    // Choose kernel: prefer CUSTOM_KERNEL_PATH, else artifact dependency
    let kernel = if let Some(custom) = std::env::var_os("CUSTOM_KERNEL_PATH") {
        let p = PathBuf::from(custom);
        let abs = std::fs::canonicalize(&p).unwrap_or(p.clone());
        println!("cargo:warning=Using custom kernel: {}", abs.display());
        // Re-run if that file changes
        println!("cargo:rerun-if-changed={}", abs.display());
        abs
    } else {
        let p = PathBuf::from(
            std::env::var_os("CARGO_BIN_FILE_LIBKERNEL")
                .expect("CARGO_BIN_FILE_LIBKERNEL not set"),
        );
        let abs = std::fs::canonicalize(&p).unwrap_or(p.clone());
        println!("cargo:warning=Using artifact kernel: {}", abs.display());
        println!("cargo:rerun-if-changed={}", abs.display());
        abs
    };

    // create an UEFI disk image (optional)
    let uefi_path = _out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();

    // create a BIOS disk image
    let bios_path = _out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel).create_disk_image(&bios_path).unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}