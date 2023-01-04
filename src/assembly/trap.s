.option norvc
.section .trap

.global asm_trap_vector
.align 4
asm_trap_vector:
	mret
