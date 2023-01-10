use self::page_table::descriptor::PageTableDescriptor;
use self::page_table::untyped::PageTableUntyped;
use crate::cpu_status::Satp;
use crate::kmem;
use crate::memory::PAGE_SIZE;
use page_table::forty_eight::SV_FORTY_EIGHT;
use page_table::thirty_nine::SV_THIRTY_NINE;
use page_table::thirty_two::SV_THIRTY_TWO;

pub use entry::EntryFlags;

pub mod entry;
pub mod page_table;

extern "C" {
    fn asm_get_satp() -> usize;
    fn asm_set_satp(_: usize);
}

/// Global that stores the type of the page table in use.
/// Provided so software can support multiple types of page tables
/// and pick between them depending on hardware support at runtime.
static mut PAGE_TABLE_TYPE: TableTypes = TableTypes::Sv39;

unsafe fn get_global_descriptor() -> &'static PageTableDescriptor {
    match unsafe { PAGE_TABLE_TYPE } {
        TableTypes::None => panic!("MMU not configured"),
        TableTypes::Sv32 => &SV_THIRTY_TWO,
        TableTypes::Sv39 => &SV_THIRTY_NINE,
        TableTypes::Sv48 => &SV_FORTY_EIGHT,
    }
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

/// Attempts to set the preferred translation table type
/// falling back if unsupported. will fall back to no
/// translation if none is supported by processor.
/// sets the satp register to the given address.
/// does not turn on address translation
fn set_translation_table(mode: TableTypes, address: &mut PageTableUntyped) -> bool {
    let mode = mode as u8;
    let address = { address as *mut _ } as usize;
    let desired = Satp::from(address, mode);

    let found = unsafe {
        asm_set_satp(desired.raw());
        asm_get_satp()
    };

    found == desired.raw()
}
