# entry point - set up machine and jump into rust code

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
	# will init then park
	# while we boot
	csrr	t0, mhartid
	bnez	t0, 4f

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
	la		ra, 5f

	# call kinit
	mret
3:
	# set up mstatus for returning to rust
	# set MPP to supervisor (11)
	# set MPIE to enabled (7)
	# set SPIE to enabled (5)
	li		t0, (0b01 << 11) | (1 << 7) | (1 << 5)
	csrw	mstatus, t0

	# set machine trap vector
	la		t2, asm_trap_vector
	csrw	mtvec, t2

	# setup MEPC to kmain as before
	la		t1, kinit_hart 
	csrw	mepc, t1

	# jump forward to park on return
	#la		ra, 5f
	mret

# init other cores - address 124
4: 
	# give each hart a stack of 64kb
	la		sp, _stack_end
	li		t0, 0x10000
	csrr	a0, mhartid
	mul		t0, t0, a0
	sub		sp, sp, t0

	# MPP machine mode (11)
	# MPIE enabled
	# SPIE enabled 
	li		t0, 0b11 << 11 | (1 << 7) | (1 <<  5)
	csrw	mstatus, t0

	# turn on MSIE (machine software interupt enable)
	li		t3, (1 << 3)
	csrw	mie, t3

	# set exception pointer to rust hart init code
	la		t1, kinit_hart
	csrw	mepc, t1

	# set up hart's trap vector
	la		t2, asm_trap_vector
	csrw	mtvec, t2

	# set return pointer so we come back here
	# when kinit_hart returns 
	la		ra, 5f

	# call kinit_hart
	mret

# initialize non-zero cores
# and park
5:
	wfi
	j		5b
