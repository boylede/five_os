.section .text

.global asm_get_misa
asm_get_misa:
    csrr   a0, misa
    ret

.global asm_get_mtvec
asm_get_mtvec:
    csrr   a0, mtvec
    ret

.global asm_set_mtvec
asm_set_mtvec:
    csrw   mtvec, a0
    ret

.global asm_get_satp
asm_get_satp:
    csrr   a0, satp
    ret

.global asm_set_satp
asm_set_satp:
    csrw   satp, a0
    ret
    