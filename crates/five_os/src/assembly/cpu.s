.option norvc
.section .text

.global asm_get_misa
asm_get_misa:
    csrr   a0, misa
    ret

.global asm_get_mvendorid
asm_get_mvendorid:
    csrr   a0, mvendorid
    ret
.global asm_get_marchid
asm_get_marchid:
    csrr   a0, marchid
    ret
.global asm_get_mimpid
asm_get_mimpid:
    csrr   a0, mimpid
    ret
.global asm_get_mhartid
asm_get_mhartid:
    csrr   a0, mhartid
    ret
.global asm_get_mstatus
asm_get_mstatus:
    csrr   a0, mstatus
    ret
.global asm_set_mstatus
asm_set_mstatus:
    csrw   mstatus, a0
    ret
.global asm_get_satp
asm_get_satp:
    csrr   a0, satp
    ret

.global asm_set_satp
asm_set_satp:
    csrw   satp, a0
    ret
    