.option norvc
.section .text

.global asm_get_misa
asm_get_misa:
    csrr   a0, misa
    ret

.global asm_read_misa_xlen
asm_read_misa_xlen:
    csrr   a0, misa
    # if a0 < 0
    bltz a0, 1f
    # so misa is > 0, meaning top bit is unset
    li a0, 32
    ret
    # at this point both top bits are unset
1:
    # topmost bit set
    srli a0, a0, 1
    bltz a0, 2f
    li a0, 64
    ret
2:
    li a0, 128
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

.global asm_get_mepc
asm_get_mepc:
    csrr   a0, mepc
    ret
.global asm_set_mepc
asm_set_mepc:
    csrw   mepc, a0
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
    