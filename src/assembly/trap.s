.option norvc
.section .text

.global asm_trap_vector
.align 4
asm_trap_vector:
.option push
.option norelax # ensures the following assembly is not relaxed by the linker
	call rust_trap
.option pop
	mret
