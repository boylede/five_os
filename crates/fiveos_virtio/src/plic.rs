use core::num::NonZeroU32;

pub const PLIC_BASE_ADDRESS: usize = 0x0c00_0000; // 0x20_0000
pub const PLIC_SIZE: usize = 0x2000; //0x20_0000 + (4 * 0x1000); // note: size is 0x1000 * num_cpus, going with 4 here due to my config
pub const PLIC_END_ADDRESS: usize = PLIC_BASE_ADDRESS + PLIC_SIZE;
const ENABLE: usize = 0x0c00_2000;
const PRIORITY: usize = 0x0c00_0000;
const PENDING: usize = 0x0c00_1000;
const THRESHOLD: usize = 0x0c20_0000;
const CLAIM_COMPLETE: usize = 0x0c20_0004;

/// ZST representing access to to the PLIC
/// todo: hide this behind some kind of
/// access control to prevent race conditions
pub struct PLIC;

impl PLIC {
    // todo: remove race condition
    pub fn enable_interrupt(&mut self, interrupt: u8) {
        let register = ENABLE as *mut u32;
        let flag = 1 << interrupt;
        unsafe {
            register.write_volatile(register.read_volatile() | flag);
        }
    }
    /// priority is a value in range 0..8
    pub fn set_priority(&mut self, id: u8, priority: u8) {
        let register = PRIORITY as *mut u32;
        let flag = (priority & 0b111) as u32;
        unsafe {
            register.add(id as usize).write_volatile(flag);
        }
    }
    /// threshold is a value in range 0..8
    pub fn set_threshold(&mut self, threshold: u8) {
        let flag = (threshold & 0b111) as u32;
        let register = THRESHOLD as *mut u32;
        unsafe {
            register.write_volatile(flag);
        }
    }
    pub fn check_pending(&self, interrupt: u8) -> bool {
        let register = PENDING as *mut u32;
        let flag = 1 << interrupt;
        let pending = unsafe { register.read_volatile() };
        pending & flag != 0
    }
    pub fn claim(&mut self) -> Option<TriggeredInterrupt> {
        let register = CLAIM_COMPLETE as *const u32;
        let number = unsafe { register.read_volatile() };
        number
            .try_into()
            .map(|number| TriggeredInterrupt { number })
            .ok()
    }
    pub fn complete(&mut self, interrupt: TriggeredInterrupt) {
        let register = CLAIM_COMPLETE as *mut u32;
        unsafe {
            register.write_volatile(interrupt.number());
        }
    }
}

pub struct TriggeredInterrupt {
    number: NonZeroU32,
}

impl TriggeredInterrupt {
    pub fn number(&self) -> u32 {
        self.number.get()
    }
}

unsafe impl Send for TriggeredInterrupt {}
unsafe impl Sync for TriggeredInterrupt {}
