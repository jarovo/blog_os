#![feature(yield_expr, gen_blocks)]

use std::{env::{VarsOs}, error::Error, path::PathBuf, process::ExitCode};

// Filter the built kernels from environment variables
fn get_kernels(env_vars: VarsOs) -> impl Iterator<Item = (String, PathBuf)> {
    gen {
        let mut kernels = env_vars.filter_map(| (k, v) | {
            k.into_string().ok().and_then(|s| {
                s.starts_with("CARGO_BIN_FILE_RUSTYK").then(|| (s, v))
            })
        });
        for (os_key, os_value) in kernels {
            let p = PathBuf::from(os_value);
            let abs = std::fs::canonicalize(&p).expect("Failed to canonicalize kernel path");
            println!("cargo:warning=Using artifact kernel: {}", abs.display());
            yield (os_key, abs)

        }
    }
}

enum ImageType {
    Bios,
    Uefi,
}

fn create_image(image_type: ImageType, kernel: &PathBuf, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    match image_type {
        ImageType::Bios => Ok(bootloader::BiosBoot::new(&kernel).create_disk_image(&path)?),
        ImageType::Uefi => Ok(bootloader::UefiBoot::new(&kernel).create_disk_image(&path)?),
    }
}


fn main() -> ExitCode {
    // set by cargo, build scripts should use this directory for output files
    let _out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    for (kernel_name, kernel_path) in get_kernels(std::env::vars_os()) {

        println!("cargo:warning=Creating images for kernel: {}", kernel_name);

        let bios_image_path = _out_dir.join(format!("{}.bios.img", kernel_name));
        let uefi_image_path = _out_dir.join(format!("{}.uefi.img", kernel_name));

        create_image(ImageType::Bios, &kernel_path, &bios_image_path).unwrap();
        create_image(ImageType::Uefi, &kernel_path, &uefi_image_path).unwrap();

        println!("cargo:warning=Created BIOS image: {}", bios_image_path.display());
        println!("cargo:warning=Created UEFI image: {}", uefi_image_path.display());

        // Declare output files as build script outputs
        println!("cargo:rerun-if-changed={}", bios_image_path.display());
        println!("cargo:rerun-if-changed={}", uefi_image_path.display()); 

        // Declare bios and uefi images as outputs
        println!("cargo:rustc-env=BIOS_IMAGE_{}={}", kernel_name, bios_image_path.display());
        println!("cargo:rustc-env=UEFI_IMAGE_{}={}", kernel_name, uefi_image_path.display());
    }

    ExitCode::SUCCESS
}