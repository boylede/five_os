.section .text

.global asm_get_misa
asm_get_misa:
    csrr   a0, misa
    ret

.global asm_get_mtvec
asm_get_mtvec:
    csrr   a0, mtvec
    ret

.global asm_get_satp
asm_get_satp:
    csrr   a0, satp
    ret