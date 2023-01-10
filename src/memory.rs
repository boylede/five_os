extern crate alloc;

use fiveos_riscv::mmu::page_table::descriptor::PageTableDescriptor;
use fiveos_riscv::mmu::page_table::forty_eight::SV_FORTY_EIGHT;
use fiveos_riscv::mmu::page_table::thirty_nine::SV_THIRTY_NINE;
use fiveos_riscv::mmu::page_table::thirty_two::SV_THIRTY_TWO;
use fiveos_riscv::mmu::{set_translation_table, TableTypes};

use crate::kmem;

pub mod allocator;

/// Global that stores the type of the page table in use.
/// Provided so software can support multiple types of page tables
/// and pick between them depending on hardware support at runtime.
static mut PAGE_TABLE_TYPE: TableTypes = TableTypes::Sv39;

pub unsafe fn get_global_descriptor() -> &'static PageTableDescriptor {
    match unsafe { PAGE_TABLE_TYPE } {
        TableTypes::None => panic!("MMU not configured"),
        TableTypes::Sv32 => &SV_THIRTY_TWO,
        TableTypes::Sv39 => &SV_THIRTY_NINE,
        TableTypes::Sv48 => &SV_FORTY_EIGHT,
    }
}

/// called in kinit
/// attempt to set the translation table to the kernel translation table,
/// and set the type of translation used.
/// panics if implementation does not support desired translation spec
/// todo: don't panic, return error or supported translation spec instead
/// todo: write PAGE_TABLE_TYPE with the resulting type
pub fn setup() {
    let kernel_page_table = kmem::get_page_table();
    if !set_translation_table(TableTypes::Sv39, kernel_page_table) {
        panic!("address translation not supported on this processor.");
    }
}
