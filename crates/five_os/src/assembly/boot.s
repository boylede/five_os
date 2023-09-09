.option norvc


.section .text.init


.global _start
_start:

.option push
.option norelax
    # load the memory location at the end of the
    # text section / begining of rodata, per layout
	la		gp, _global_pointer
.option pop

	# clear address translation
	# technically not on anyway
	csrw	satp, zero

	# cores other than 0
	# will park while we boot
	csrr	t0, mhartid
	bnez	t0, 3f

	# zero bss section
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f

1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b

	# done zero-ing bss
2:
	# setup stack pointer per layout
	la		sp, _stack_end

	# set previous MPP to Machine
	# so mret won't pop us out of Machine privilege
	# when we go to kinit
	li		t0, 0b11 << 11
	csrw	mstatus, t0

	# disable interupts
	csrw	mie, zero

	# set MEPC to kinit so we will return there
	# on mret
	la		t1, kinit
	csrw	mepc, t1

	# set return pointer so we come back here
	# when kinit returns 
	la		ra, 2f

	# call kinit
	mret
2:
	# again, set MPP to machine
	# but also MIE and MPIE to 1
	# to enable machine interrupts,
	# before and after mret
	li		t0, (11 << 11) | (1 << 7) | (1 << 5)
	csrw	mstatus, t0

	# set machine trap vector
	la		t2, asm_trap_vector
	csrw	mtvec, t2

	# setup MEPC to kmain as before
	la		t1, kmain 
	csrw	mepc, t1

	# enable external, timer, and software 
	# interrupts only
	li		t2, 0x888
	csrw	mie, t2

	# set return pointer so we come back here
	# when kmain returns 
	la		ra, 3f

	# call kmain
	mret

# initialize non-zero cores
# and park
3:
	wfi
	j		3b
