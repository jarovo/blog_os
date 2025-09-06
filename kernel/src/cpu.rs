// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Architectural processor code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::arch_cpu

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------
use qemu_exit::QEMUExit;

const QEMU_EXIT_HANDLE: qemu_exit::X86 = qemu_exit::X86::new(0xf4, 0x11);

/// Make the host QEMU binary execute `exit(1)`.
pub fn qemu_exit_failure() -> ! {
    QEMU_EXIT_HANDLE.exit_failure()
}

/// Make the host QEMU binary execute `exit(0)`.
pub fn qemu_exit_success() -> ! {
    QEMU_EXIT_HANDLE.exit_success()
}