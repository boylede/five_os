.section .text
.global asm_get_misa
asm_get_misa:
    csrr   a0, misa
    ret
