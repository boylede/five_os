# binutils commands

cargo-strip --bin=five_os --release -- --strip-all -o smol-five

cargo readobj --bin=five_os -- -file-headers

cargo nm --release

cargo objdump --release --bin=five_os -- -disassemble -no-show-raw-insn > disasm.txt

cargo size --release --bin=five_os -- -A -x


# qemu & gbd commands


run latest debug build in qemu, but pause it and start gbd server:
qemu-system-riscv64 -machine virt -cpu rv64 -d guest_errors,unimp -smp 4 -m 128M -serial mon:stdio -bios none -display none -device virtio-rng-device -device virtio-gpu-device -device virtio-net-device -device virtio-tablet-device -device virtio-keyboard-device -s -S -kernel ./target/riscv64gc-unknown-none-elf/debug/five_os

run gbd connection:
riscv64-unknown-elf-gdb ./target/riscv64gc-unknown-none-elf/debug/five_os

type target extended-remote :1234

