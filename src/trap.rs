use core::ptr::null_mut;

use crate::layout::StaticLayout;
use crate::{abort, print, println};

/// Context information collected in trap.s before calling rust trap handler
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    /// General purpose registers
    pub regs: [usize; 32],
    /// Floating point purpose registers
    pub fregs: [usize; 32],
    /// satp
    pub satp: usize,
    /// stack pointer
    pub trap_stack: *mut u8,
    /// core id
    pub hartid: usize,
}

impl TrapFrame {
    const NULL: TrapFrame = TrapFrame {
        regs: [0; 32],
        fregs: [0; 32],
        satp: 0,
        trap_stack: null_mut(),
        hartid: 0,
    };
}

/// Global store of trapframes, one per core/hart. we're providing for 4 harts here.
/// todo: initialize in kinit with accurate number of harts
pub static mut GLOBAL_TRAPFRAMES: &mut [TrapFrame; 4] = &mut [TrapFrame::NULL; 4];

#[derive(Debug)]
#[repr(transparent)]
struct TrapCause(usize);

impl TrapCause {
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
}

#[repr(usize)]
#[non_exhaustive]
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
    // Reserved = 10,
    EnvironmentCallMmode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    // Reserved = 14,
    StoreAMOPageFault = 15,
}

#[no_mangle]
#[repr(align(4))]
extern "C" fn rust_trap(
    epc: usize,
    tval: usize,
    acause: usize,
    hart: usize,
    status: usize,
    frame: &mut TrapFrame,
) -> usize {
    let cause = TrapCause(acause);
    let mut return_pc = epc;
    let cause_number = cause.number();
    println!("entered trap handler with cause {}", cause_number);
    if cause.is_async() {
        match cause_number {
            3 => {
                println!("Machine software interrupt CPU#{}", hart);
            }
            7 => {
                println!("Machine timer interrupt CPU#{}", hart);
            }
            11 => {
                println!("Machine external interrupt CPU#{}", hart);
            }
            _ => {
                panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_number);
            }
        }
    } else {
        match cause_number {
            0 => panic!(
                "Instruction address misaligned: #{}/0x{:08x}/{}",
                hart, epc, tval
            ),
            1 => panic!("Instruction access fault: #{}/0x{:08x}/{}", hart, epc, tval),
            2 => panic!("Illegal instruction: #{}/0x{:08x}/{}", hart, epc, tval),
            3 => {
                println!("Skipped breakpoint: #{}/0x{:08x}", hart, epc);
                return_pc += 4;
            }
            4 => panic!("Load address misaligned: #{}/0x{:08x}/{}", hart, epc, tval),
            5 => panic!("Load access fault: #{}/0x{:08x}/{}", hart, epc, tval),
            6 => panic!(
                "Store/AMO address misaligned: #{}/0x{:08x}/{}",
                hart, epc, tval
            ),
            7 => panic!("Store/AMO fault: #{}/0x{:08x}/{}", hart, epc, tval),
            8 => {
                println!("External call from user mode: #{}/0x{:08x}", hart, epc);
                return_pc += 4;
            }
            9 => {
                println!("External call from supervisor mode:#{}/0x{:08x}", hart, epc);
                return_pc += 4;
            }
            11 => {
                println!("External call from Machine mode?!:#{}/0x{:08x}", hart, epc);
                return_pc += 4;
            }

            12 => {
                panic!("Instruction page fault: #{}/0x{:08x}/{}", hart, epc, tval);
            }
            13 => {
                panic!("Load page fault: #{}/0x{:08x}/{}", hart, epc, tval);
            }
            15 => {
                panic!("Store page fault: #{}/0x{:08x}/{}", hart, epc, tval);
            }
            _ => {
                panic!("Unhandled synchronous trap: #{}/{}", hart, cause_number);
            }
        }
    };
    return_pc
}

pub fn setup_trap_handler() {
    let layout = StaticLayout::get();
    let _ts = layout.trap_start;
    // TODO: set up trap handler
}
