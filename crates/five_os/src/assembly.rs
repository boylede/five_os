use core::arch::global_asm;

global_asm!(include_str!("assembly/boot.s"));
global_asm!(include_str!("assembly/cpu.s"));
global_asm!(include_str!("assembly/trap.s"));
