pub fn boot_qemu(disk_image_path: &str) -> Result<(), std::io::Error> {
    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    
    println!("Using BIOS image: {}", disk_image_path);
    cmd.arg("-drive").arg(format!("if=virtio,format=raw,readonly=on,file={disk_image_path}"));

    cmd.arg("-device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-serial").arg("stdio");

    if std::env::var_os("QEMU_GDB").is_some() {
        eprintln!("QEMU GDB stub enabled on :1234 (CPU paused)");
        cmd.args(["-S", "-gdb", "tcp::1234"]);
        cmd.args(["-no-reboot", "-no-shutdown"]);
        cmd.args(["-d", "int,guest_errors,cpu_reset"]);
        cmd.args(["-accel", "tcg"]);

    }

    print!("Starting QEMU: {:?}\n", cmd);
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    if status.code() == Some(0x11) {
        println!("QEMU exited with success.");
        Ok(())
    } else {
        eprintln!("QEMU exited with status: {}", status);
        Err(std::io::Error::new(std::io::ErrorKind::Other, "QEMU failed"))
    }
}