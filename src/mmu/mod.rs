use crate::layout::StaticLayout;
use crate::page::{align_address, PAGE_SIZE};
use crate::{print, println};

mod thirty_nine;
mod thirty_two;

pub use thirty_nine::Sv39Table;
pub use thirty_two::Sv32Table;

/// Abstraction over any MMU-backed page table type
pub struct PageTable(Page);

pub enum TableTypes {
    Sv32(Sv32Table),
    Sv39(Sv39Table),
    // Sv48. // todo
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

/// Trait for describing MMU-backed page table entries. Each page-table mode (Sv32, Sv39,
/// Sv48, etc) provides an implementation. Code outside of this module should not need to
/// depend on a specific page-table mode.
trait PtEntry {
    type Entry;
    /// Checks if the entry is valid
    fn valid(&self) -> bool;
    /// Inserts given address into entry, sets valid bit and clears all other flags
    fn put_address(&mut self, physical: usize);
    /// Inserts the given address, sets valid bit, sets permissions, and clears other bits
    fn put_entry(&mut self, physical: usize, permissions: PermFlags);
    /// Clears the entry and sets each parameter, as well as the valid flag. Clears reserved bits.
    fn set(
        &mut self,
        permissions: PermFlags,
        user: bool,
        global: bool,
        soft: SoftFlags,
        physical: usize,
    );
    /// Returns current software flags state
    fn software_flags(&self) -> SoftFlags;
    /// Allows the software bits to be set
    fn set_software(&mut self, value: SoftFlags);
    /// Checks if this is considered a leaf entry
    fn leaf(&self) -> bool;
    fn descend(&self, virt: usize) -> &mut Self::Entry;
}

trait Table {
    type Entry;
    type Address;
    fn len() -> usize;
    fn initialize(base_address: usize) -> Self;
    fn get_physical_address(&self, address: usize) -> usize;
}

trait Address {
    fn to_physical(&self) -> usize;
}

pub fn setup() -> *mut Sv39Table {
    let layout = StaticLayout::get();
    println!("heap_start is {:x}", layout.heap_start);
    let mut satp = crate::cpu_status::Satp::from_address(layout.heap_start);
    println!(
        "resulting address is {:x}",
        align_address(layout.heap_start)
    );
    satp.set_mode(8);
    crate::cpu_status::set_satp(&satp);
    let table = Sv39Table::at_address(satp.address());
    table
}

pub fn test() {
    assert!(core::mem::size_of::<thirty_two::Sv32Entry>() == 4);
    assert!(core::mem::size_of::<thirty_nine::Sv39Entry>() == 8);
}
