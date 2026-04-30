use std::{fs::OpenOptions};
use anyhow::Result;

use crate::models::{Image, BootKind};


pub struct QemuVirtualMachine {
    disk_image_path: String,
    graphic: bool,
    boot_kind: BootKind,
    qemu_gdb: Option<u16>,
}

impl QemuVirtualMachine {
    pub fn builder() -> QemuVirtualMachineBuilder {
        QemuVirtualMachineBuilder::default()
    }
    
    fn prepare_qemu_cmd(self) -> Result<std::process::Command> {
        let mut cmd = std::process::Command::new("qemu-system-x86_64");
        
        println!("Using image: {}", self.disk_image_path);
        if let Ok(_) = OpenOptions::new().write(true)
                                .read(true)
                                .open("/dev/kvm") {
            cmd.arg("-enable-kvm");
        }
        match self.graphic {
            true => {
                cmd.arg("-serial").arg("stdio");
            }
            false => {
                cmd.arg("-nographic");
            }
        }
    
        cmd.arg("-drive").arg(format!("if=virtio,format=raw,readonly=on,file={}", self.disk_image_path));
        
        cmd.arg("-device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

        if let Some(gdb_port) = self.qemu_gdb {
            eprintln!("QEMU GDB stub enabled on :{} (CPU paused)", gdb_port);
            cmd.args(["-S", "-gdb", &format!("tcp::{}", gdb_port)]);
            cmd.args(["-no-reboot", "-no-shutdown"]);
            cmd.args(["-d", "int,guest_errors,cpu_reset"]);
            cmd.args(["-accel", "tcg"]);
        }

        match self.boot_kind {
            BootKind::Uefi => {
                cmd = QemuVirtualMachine::append_ovmf_drives(cmd)?;
            }
            BootKind::Bios => {}
        }

        Ok(cmd)
    }

    fn append_ovmf_drives(mut cmd: std::process::Command) -> Result<std::process::Command> {
        let ovmf_prefix = std::env::var("OVMF_PREFIX")?;
        cmd.arg("-drive").arg(format!("if=pflash,format=raw,readonly=on,file={}/OVMF_CODE.fd", ovmf_prefix));
        cmd.arg("-drive").arg(format!("if=pflash,format=raw,readonly=on,file={}/OVMF_VARS.fd", ovmf_prefix));
        Ok(cmd)
    }
    
    pub fn run(self) -> Result<()> {
        let mut cmd = self.prepare_qemu_cmd()?;

        print!("Starting QEMU: {:?}\n", cmd);
        let mut child = cmd.spawn()?;

        let status = child.wait()?;
        if status.code() == Some(0x11) {
            println!("QEMU exited with success.");
            Ok(())
        } else {
            eprintln!("QEMU exited with status: {}", status);
            Err(anyhow::anyhow!("QEMU failed"))
        }
    }
}

pub struct QemuVirtualMachineBuilder {
    disk_image_path: Option<String>,
    boot_kind: BootKind,
    graphic: bool,
    qemu_gdb: Option<u16>,
}

impl Default for QemuVirtualMachineBuilder {
    fn default() -> Self {
        Self {
            graphic: false,
            qemu_gdb: None,
            disk_image_path: None,
            boot_kind: BootKind::Bios,
        }
    }
}

impl QemuVirtualMachineBuilder {
    pub fn with_bootable_image(mut self, image: &Image) -> Self {
        self.disk_image_path = Some(image.image_file_path.to_string_lossy().into());
        self.boot_kind = image.boot_kind.clone();
        self
    }

    pub fn with_graphic(mut self, graphic: bool) -> Self {
        self.graphic = graphic;
        self
    }

    pub fn with_qemu_gdb(mut self, qemu_gdb: Option<u16>) -> Self {
        self.qemu_gdb = qemu_gdb;
        self
    }

    pub fn build(self) -> QemuVirtualMachine {
        QemuVirtualMachine {
            disk_image_path: self.disk_image_path.expect("The disk_image_path must be set."),
            graphic: self.graphic,
            boot_kind: self.boot_kind,
            qemu_gdb: self.qemu_gdb,
        }
    }
}