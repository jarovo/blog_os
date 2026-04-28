use anyhow::Result;
use clap::{Command, arg};
use rustyk_build::qemu_virtual_machine::QemuVirtualMachine;

fn cli() -> Command {
    Command::new("rustyk")
        .about("Run Rustyk in QEMU Virtual Machine.")
        .arg(
            arg!(-b --boot <BOOT_KIND>)
            .help("The boot kind to use.")
            .value_parser(clap::value_parser!(rustyk_build::models::BootKind))
            .default_value("bios"))
        .arg(
            arg!(-d --dbg <DEBUG_PORT>)
            .value_parser(clap::value_parser!(u16))
            .help("The debug port to use.")
        )

        .subcommand(
            Command::new("run-image")
            .arg(
                arg!(-i --image <KERNEL_IMAGE>)
                .help("The kernel image to boot.")
                .default_value("kernel"))
        )
        .subcommand(
            Command::new("run-tests")
        )
}
    

fn main() -> Result<()> {
    let build = rustyk_build::models::Build::read_info().expect("Expected to read build info");
    
    let matches = cli().get_matches();
    let boot_kind = matches.get_one::<rustyk_build::models::BootKind>("boot").expect("required").clone();
    let debug_port = match matches.get_one::<u16>("dbg") {
        Some(port) => Some(*port),
        None => None,
    };

    match matches.subcommand() {
        Some(("run-image", sub_matches)) => {
            let kernel_image = sub_matches.get_one::<String>("image").expect("required");
            let bootable_image = build.images().iter()
                .find(|image|
                    image.kernel_artifact.declaration.name == *kernel_image
                    && image.kernel_artifact.declaration.dep == "rustyk"
                    && image.boot_kind == boot_kind)
                .ok_or_else(|| anyhow::anyhow!("No matching image found for kernel: {}, boot kind: {:?}", kernel_image, boot_kind))?;
            
            QemuVirtualMachine::builder()
                .with_bootable_image(bootable_image)
                .with_qemu_gdb(debug_port)
                .with_graphic(true)
                .build().run()?;
        },
        Some(("run-tests", _sub_matches)) => {
            build.images().iter()
                .filter(|image|
                    image.kernel_artifact.declaration.dep == "rustyk"
                    && image.kernel_artifact.declaration.name.starts_with("test_")
                    && image.boot_kind == boot_kind)
                .try_for_each(|image| QemuVirtualMachine::builder()
                    .with_bootable_image(image)
                    .with_qemu_gdb(debug_port)
                    .with_graphic(true)
                    .build()
                    .run())?;
        }
        None => {
            println!("No subcommand was used. Use --help for more information.");
        }
        Some((cmd, _)) => {
            println!("Unknown subcommand: {}. Use --help for more information.", cmd);
        }
    }
    Ok(())
}