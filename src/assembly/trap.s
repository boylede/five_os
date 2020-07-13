.section .text
.option norvc
.section .trap
.global asm_trap_vector
.align 4
asm_trap_vector:
    # todo: complete
    addi a0, a0, 0
    addi a0, a0, 0
    addi a0, a0, 0
    addi a0, a0, 0
    addi a0, a0, 0
    addi a0, a0, 0
    addi a0, a0, 0
    addi a0, a0, 0
    mret
