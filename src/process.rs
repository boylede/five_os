use fiveos_riscv::mmu::page_table::descriptor::PageTableDescriptor;
use fiveos_riscv::mmu::page_table::untyped::PageTableUntyped;
use fiveos_riscv::mmu::page_table::PAGE_SIZE;
use fiveos_riscv::mmu::EntryFlags;

use crate::trap::TrapFrame;

// todo: replace with atomic increment
static mut PID_COUNTER: u32 = 0;

/// size of stack for new processes, in pages
const STACK_SIZE: usize = 1;

/// The running status of a process
#[derive(Clone, Copy)]
pub enum Status {
    Created,
    Waiting,
    Running,
    Blocked,
    Terminated,
    SwapWait,
    SwapBlock,
}

#[derive(Clone)]
pub struct Process {
    id: u32,
    state: Status,
    instruction_pointer: usize,
    stack_pointer: *mut u8,
    page_table: *mut u8,
    trap_frame: TrapFrame,
}

impl Process {
    pub fn new(func: fn(), descriptor: &PageTableDescriptor) -> Option<Process> {
        todo!()
    }
}
