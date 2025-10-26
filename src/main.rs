// src/main.rs

use std::path::PathBuf;


fn run(vm_name: &str, bios_path: &str) -> Result<(), std::io::Error> {
    let mut cmd = std::process::Command::new("virt-install");
    let log_path= PathBuf::from(format!("/tmp/{}.serial.log", vm_name));
    cmd.args(&[
        "--name", vm_name,
        "--serial", format!("file,path={}", log_path.display()).as_str(),
        "--transient",
        "--arch", "x86_64",
        "--osinfo", "unknown",
        "--memory", "1024",
        "--import", "--disk", bios_path,
        "--security", "type=none",
        "--qemu-commandline", "--device isa-debug-exit,iobase=0xf4,iosize=0x04",
    ]);
    cmd.spawn()?.wait()?.success().then(|| { check_serial_log(log_path) }).unwrap_or_else(|| {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "VM execution failed"))
    })
}

fn check_serial_log(log_path: PathBuf) -> Result<(), std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);

    
    for line in reader.lines() {
        let line = line?;
        if line.contains("OK1234") {
            println!("Test passed: found OK1234 in serial log.");
            return Ok(());
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::Other, "Test failed: OK1234 not found in serial log."))
}

fn main() {
    run("rustyk_vm", env!("BIOS_IMAGE_CARGO_BIN_FILE_RUSTYK_kernel")).unwrap();
}


#[cfg(test)]
mod tests {

    #[test]
    fn heap_allocation() {
        super::run("rustyk_test_vm_heap_allocation",
        env!("BIOS_IMAGE_CARGO_BIN_FILE_RUSTYK_test_heap_allocation")).unwrap();
    }

    #[test]
    fn stack_overflow() {
        super::run("rustyk_test_vm_stack_overflow",
        env!("BIOS_IMAGE_CARGO_BIN_FILE_RUSTYK_test_stack_overflow")).unwrap();
    }
}