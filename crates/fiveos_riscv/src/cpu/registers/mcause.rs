use core::{arch::asm, mem};

const XLEN: usize = mem::size_of::<usize>() * 8;
const KINDMASK: usize = 0b1 << (XLEN - 1);

pub enum TrapKind {
    Interrupt,
    Exception,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TrapCause(pub usize);

impl TrapCause {
    pub fn read() -> TrapCause {
        TrapCause(get())
    }
    pub fn is_async(&self) -> bool {
        (self.0 >> 63) & 1 == 1
    }
    pub fn number(&self) -> usize {
        self.0 & 0xfff
    }
    pub fn is_software(&self) -> bool {
        let n = self.number();
        !self.is_async() && n <= 3
    }
    pub fn is_timer(&self) -> bool {
        let n = self.number();
        !self.is_async() && n > 3 && n <= 7
    }
    pub fn is_external(&self) -> bool {
        let n = self.number();
        !self.is_async() && n > 7 && n <= 11
    }
    pub fn is_user(&self) -> bool {
        let n = self.number();
        n == 0 || n == 4 || n == 8
    }
    pub fn is_supervisor(&self) -> bool {
        let n = self.number();
        n == 1 || n == 5 || n == 9
    }
    pub fn is_machine(&self) -> bool {
        let n = self.number();
        n == 3 || n == 7 || n == 11
    }
    pub fn known_cause(&self) -> KnownCause {
        todo!()
    }
}

/// An enumeration over parsed values of mcause
/// unexpected values are filtered out into Reserved values
pub enum KnownCause {
    Sync(SyncCause),
    Async(AsyncCause),
}

/// enumeration for defined sync causes
/// 2, 6, 10, 12, 13, 14, 15 are reserved in this version of the spec
/// causes >=16 are implemention defined
pub enum SyncCause {
    UserSoftwareInterrupt = 0,
    SupervisorSoftwareInterrupt = 1,
    Reserved = 2,
    MachineSoftwareInterrupt = 3,
    UserTimerInterrupt = 4,
    SupervisorTimerInterrupt = 5,
    MachineTimerInterrupt = 7,
    UserExternalInterrupt = 8,
    SupervisorExternalInterrupt = 9,
    MachineExternalInterrupt = 11,
    ImplementationDefinedLocalInterrupts = 16,
}

/// enumeration for defined async causes
/// 10, 14, >=16 are reserved in this version of the spec
pub enum AsyncCause {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreAMOAddressMisaligned = 6,
    StoreAMOAccessFault = 7,
    EnvironmentCallUmode = 8,
    EnvironmentCallSmode = 9,
    EnvironmentCallMmode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StoreAMOPageFault = 15,
    Reserved = 254,
}

/// Gets the kind of trap triggered, an interrupt or an exception.
#[inline]
pub fn kind() -> TrapKind {
    let test: usize;
    unsafe { asm!("csrr {tmp}, mcause", tmp = out(reg) test) };
    match test & KINDMASK == 0 {
        true => TrapKind::Exception,
        false => TrapKind::Interrupt,
    }
}

/// Gets the code from mcause
#[inline]
pub fn code() -> usize {
    let out: usize;
    unsafe { asm!("csrr {tmp}, mcause", tmp = out(reg) out) };
    out & !KINDMASK
}

/// Gets the raw mcause value
#[inline]
pub fn get() -> usize {
    let out: usize;
    unsafe { asm!("csrr {tmp}, mcause", tmp = out(reg) out) };
    out
}

/// attempt to set mcause
#[inline]
pub unsafe fn set(value: usize) {
    unsafe { asm!("csrw mcause, {tmp}", tmp = in(reg) value) };
}
