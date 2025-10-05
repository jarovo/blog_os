#!/bin/bash

# Build the kernel with test features
echo "Building kernel with tests..."
cd kernel
cargo build --bin libkernel --target x86_64-unknown-none --features with-tests

if [ $? -ne 0 ]; then
    echo "Kernel build failed!"
    exit 1
fi

# Go back to root and set the kernel path
cd ..
export CUSTOM_KERNEL_PATH=$(pwd)/target/x86_64-unknown-none/debug/libkernel

# Build and run the bootloader with the test kernel
echo "Running kernel tests in QEMU..."
cargo run

echo "Kernel tests completed."