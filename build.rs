use std::{process::ExitCode};
use rustyk_build::models::Build;

fn main() -> ExitCode {
    let dep = "rustyk";

    // Print build environment vars.
    print!("cargo:info=Build environment variables:\n");
    for (key, value) in std::env::vars() {
        print!("cargo:info=  {}={}\n", key, value);
    }

    const ARTIFACTS: [&str; 3] = [ "kernel", "test_heap_allocation", "test_stack_overflow" ];
    let build = Build::new(dep, &ARTIFACTS);
    build.write_info().expect("Expected to write build info");
    ExitCode::SUCCESS
}