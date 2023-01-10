use crate::cpu::status::Satp;

use self::page_table::descriptor::PageTableDescriptor;
use self::page_table::untyped::PageTableUntyped;
use self::page_table::{PAGE_ADDR_MAGNITIDE, PAGE_SIZE};

pub use entry::EntryFlags;

pub mod entry;
pub mod page_table;
pub mod physical_address;
pub mod virtual_address;

extern "C" {
    fn asm_get_satp() -> usize;
    fn asm_set_satp(_: usize);
}

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
#[repr(align(4096))]
pub struct Page(pub [u8; PAGE_SIZE]);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PageSize {
    Page = 0,
    Megapage = 1,
    GigaPage = 2,
    TeraPage = 3,
}

impl PageSize {
    pub fn to_level(&self) -> usize {
        *self as usize
    }
}

/// Attempts to set the preferred translation table type
/// falling back if unsupported. will fall back to no
/// translation if none is supported by processor.
/// sets the satp register to the given address.
/// does not turn on address translation
pub fn set_translation_table(mode: TableTypes, address: &mut PageTableUntyped) -> bool {
    let mode = mode as u8;
    let address = { address as *mut _ } as usize;
    let desired = Satp::from(address, mode);

    let found = unsafe {
        asm_set_satp(desired.raw());
        asm_get_satp()
    };

    found == desired.raw()
}

/// Produces a page-aligned address by adding one
/// less than the page size (4095), then masking low bits
/// to decrease the address back to the nearest page boundary
pub const fn align_address_to_page(address: usize) -> usize {
    align_power(address, PAGE_ADDR_MAGNITIDE)
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
pub const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that the number of low bits equal to power is set to zero.
pub const fn align_power(address: usize, power: usize) -> usize {
    align_to(address, 1 << power)
}
