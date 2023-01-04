# binutils commands

cargo-strip --bin=five_os --release -- --strip-all -o smol-five

cargo readobj --bin=five_os -- -file-headers

cargo nm --release

cargo objdump --release --bin=five_os -- -disassemble -no-show-raw-insn > disasm.txt

cargo size --release --bin=five_os -- -A -x
