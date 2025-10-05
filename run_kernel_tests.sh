#!/bin/bash
set -euo pipefail

TEST_NAME="${1:-}"

echo "Building kernel/test..."
if [[ -n "${TEST_NAME}" ]]; then
  (cd kernel && cargo test --test "$TEST_NAME" --target x86_64-unknown-none --no-run)
  export CUSTOM_KERNEL_PATH="$(readlink -f "$(find target/x86_64-unknown-none/debug/deps -type f -executable -name "${TEST_NAME}-*" | head -n1)")"
else
  (cd kernel && cargo build --bin libkernel --target x86_64-unknown-none --features with-tests)
  export CUSTOM_KERNEL_PATH="$(readlink -f "target/x86_64-unknown-none/debug/libkernel")"
fi

echo "Using kernel ELF: ${CUSTOM_KERNEL_PATH}"
cargo run
echo "Kernel tests completed."