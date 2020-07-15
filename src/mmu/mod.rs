use crate::layout::StaticLayout;
use crate::page::{align_address, PAGE_SIZE};
use crate::{print, println};

mod thirty_nine;
mod thirty_two;


/// Abstraction over any MMU-backed page table type
pub struct PageTable(Page);

/// The different types of page tables possible in Riscv
/// for both 32 bit and 64bit systems
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TableTypes {
    None = 0,

    Sv32 = 1, // 32-bit only

    // 64-bit only
    Sv39 = 8,
    Sv48 = 9,
}

pub struct Page([u8; PAGE_SIZE]);

#[repr(C)]
#[derive(Clone, Copy)]
pub enum PermFlags {
    Leaf = 0b000,
    ReadOnly = 0b001,
    ReadWrite = 0b011,
    ReadWriteExecute = 0b111,
    ReadExecute = 0b101,
}

/// Placeholder for the 2 software-defined bits allowed in the mmu's page table entries
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SoftFlags(u8);

    let layout = StaticLayout::get();
}
