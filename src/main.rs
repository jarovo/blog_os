// src/main.rs

use ovmf_prebuilt::{Arch, FileType, Source, Prebuilt};

fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");
    
    // choose whether to start the UEFI or BIOS image
    let uefi = false;


    let prebuilt = Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("failed to update prebuilt");
    let uefi_bios = prebuilt.get_file(Arch::X64, FileType::Code);

    let mut cmd = std::process::Command::new("/usr/libexec/qemu-kvm");
    // let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        println!("Using UEFI bios: {}", uefi_bios.display());
        println!("Using UEFI image: {}", uefi_path);
        cmd.arg("-bios").arg(uefi_bios);
        cmd.arg("-drive").arg(format!("if=virtio,format=raw,readonly=on,file={uefi_path}"));
    } else {
        println!("Using BIOS image: {}", bios_path);
        cmd.arg("-drive").arg(format!("if=virtio,format=raw,readonly=on,file={bios_path}"));        
    }
    cmd.arg("-device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-serial").arg("stdio");

    print!("Starting QEMU: {:?}\n", cmd);
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}