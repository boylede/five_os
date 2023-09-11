use core::{arch::asm, mem};

const XLEN: usize = mem::size_of::<usize>() * 8;
const KINDMASK: usize = 0b1 << (XLEN - 1);

pub enum TrapKind {
    Interrupt,
    Exception,
}

/// Gets the kind of trap triggered, an interrupt or an exception.
pub fn kind() -> TrapKind {
    let test:usize;
    unsafe {asm!("csrr {tmp}, mcause", tmp = out(reg) test)};
    match test & KINDMASK == 0 {
        true => TrapKind::Exception,
        false => TrapKind::Interrupt,
    }
}

/// Gets the code from mcause
pub fn code() -> usize {
    let out:usize;
    unsafe {asm!("csrr {tmp}, mcause", tmp = out(reg) out)};
    out & !KINDMASK
}

/// Gets the raw mcause value
pub fn get() -> usize {
    let out:usize;
    unsafe {asm!("csrr {tmp}, mcause", tmp = out(reg) out)};
    out
}

/// attempt to set mcause
pub unsafe fn set(value: usize) {
    unsafe {asm!("csrw mcause, {tmp}", tmp = in(reg) value)};
}
