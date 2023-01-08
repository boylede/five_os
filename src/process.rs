use crate::mem::page::alloc;
use crate::mmu::{PageTable, EntryFlags};
use crate::{
    mem::{page::zalloc, PAGE_SIZE},
    trap::TrapFrame,
};

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
    pub fn new(func: fn()) -> Option<Process> {
        let func = func as usize;
        let id = unsafe {
            let id = PID_COUNTER;
            PID_COUNTER += 1;
            id
        };
        let stack = alloc(STACK_SIZE)? as *mut u8;
        let stack_pointer = unsafe { stack.add(PAGE_SIZE * STACK_SIZE) };
        let page_table = zalloc(1)? as *mut PageTable;
        unsafe {
            let table = page_table.as_mut().unwrap();
            table.map(0,0,0,EntryFlags::USER_READ_WRITE);
        }


        let mut trap_frame = TrapFrame::zero();
        trap_frame.regs[2] = stack_pointer as usize;
        
        
        let page_table = page_table as *mut u8;
        let process = Process {
            id,
            state: Status::Created,
            instruction_pointer: func,
            stack_pointer,
            page_table,
            trap_frame,
        };

        Some(process)
    }
}
