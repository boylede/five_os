//! Machine Trap Vector 

use core::arch::asm;

#[inline]
pub unsafe fn set_trap_vector(handler: fn()) {
    let address = handler as *const fn() as usize;
    assert!(address & 0b11 == 0, "Attempted to set trap vector to misaligned pointer");
    asm!("csrw mtvec, {tmp}", tmp = in(reg) address);
}

#[inline]
pub unsafe fn set_mode(bool: bool) {
    if bool {
        asm!("csrc mtvec, {tmp}", tmp = in(reg) 0b1);
    } else {
        asm!("csrs mtvec, {tmp}", tmp = in(reg) 0b1);
    }
}

#[inline]
pub unsafe fn set_low_bits(value: u8) {
    assert!(value & !0b11 == 0, "Attempted to set values outside of mtvec low bits");
    asm!("csrc mtvec, {tmp}", "csrs mtvec, {v}", tmp = in(reg) 0b11, v = in(reg) value & 0b11);
}
