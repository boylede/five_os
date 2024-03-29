.option push
.option norvc
.section .text

.altmacro
.set NUM_GP_REGS, 32
.set NUM_FP_REGS, 32
.set REG_SIZE, 8
.set MAX_CPUS, 8

.macro save_gp i, basereg=t6
	sd	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
	ld	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro save_fp i, basereg=t6
	fsd	f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm
.macro load_fp i, basereg=t6
	fld	f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm


.align 4
.global asm_trap_vector
asm_trap_vector:
.option push
.option norelax # ensures the following assembly is not relaxed by the linker
	# swap csr with t6
	csrrw	t6, mscratch, t6

.set i, 1
.rept 30
	save_gp %i
	.set i, i+1
.endr

	mv t5, t6
	csrr t6, mscratch
	save_gp 31, t5

	csrw mscratch, t5
# load rust_trap arguments
	csrr	a0, mepc
	csrr	a1, mtval
	csrr	a2, mcause
	csrr	a3, mhartid
	csrr	a4, mstatus
	mv		a5, t5
	ld		sp, 520(a5)
	call rust_trap

# restore registers and return
	csrw	mepc, a0
	csrr	t6, mscratch
.set	i, 1
.rept	31
	load_gp %i
	.set	i, i+1
.endr

.option pop
.global trapbreak
trapbreak:
	mret

.option pop
