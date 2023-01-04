# don't use compressed instructions
.option norvc
.section .data

.section .text.init
.global _start
_start:
    # if core id != 0, jump to infinite loop
    # we only run boot sequence on core 0,
    # other cores will be initialized later
    csrr    t0, mhartid
    bnez    t0, 4f
    # clear address translation, protection
    # this is not needed, implementation should set this zero already
    # more of a reference for the reader
    csrw    satp, zero


.option push
.option norelax # ensures the following assembly is not relaxed by the linker
    # load the memory location at the end of the
    # text section / begining of rodata, per layout
    la  gp, _global_pointer
.option pop # reverts the norelax option to whatever was set before

    # clear .bss section
    la      a0, _bss_start
    la      a1, _bss_end
    bgeu    a0, a1, 2f
1:
    sd      zero, (a0)
    addi    a0, a0, 8
    bltu    a0, a1, 1b
2:
    # allow superviser-mode interrupt and exception handling
    li      t5, 0xffff
    csrw    medeleg, t5
    csrw    mideleg, t5

    # set the stack pointer to end of stack space
    la      sp, _stack_end
    # set the mstatus register, MPP=3, MPIE=1, MIE=1
    # li      t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    li      t0, (0b11 << 11)
    csrw    mstatus, t0
    # set MEPC (exception program counter) to kernel entry point
    la      t1, kinit
    csrw    mepc, t1

    # li      t3, (1 << 3) | (1 << 7) | (1 << 11)
    # csrw    mie, t3
    csrw    mie, zero

    # set the address we will return to after kinit runs
    la      ra, enter_supervisor
    # mret will update mstatus as well
    mret


# setup to run after kinit has finished
enter_supervisor:
    # set mstatus previous-status bits to desired status
    # MPP = 1 [supervisor], offset 11; mpie = 1, offset 7, spie = 1, offset 5
    li		t0, (0b01 << 11) | (1 << 7) | (1 << 5)
	csrw	mstatus, t0
    # set trap vector
    la		t2, asm_trap_vector
	csrw	mtvec, t2
    # set mepc to the desired rust kernel main function, kmain
    la		t1, kmain
	csrw	mepc, t1

    # set mie
    # li		t2, (1 << 1) | (1 << 5) | (1 << 9)
	# csrw	mie, t2
    li      t3, (1 << 3) | (1 << 7) | (1 << 11)
    csrw    mie, t3
    # set ra to where we will come back to when kmain finishes.
    la      ra, 4f
    # use mret, which moves previous-status to current status bits
    # and moves instruction pointer back into rust code
    mret


# infinite loop
4:
    wfi
    j   4b
