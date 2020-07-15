use crate::cpu_status::Satp;
use crate::kmem;
use crate::layout::StaticLayout;
use crate::page::{align_address, zalloc, PAGE_SIZE};
use crate::{print, println};

mod thirty_nine;
mod thirty_two;

pub use thirty_nine::SV_THIRTY_NINE;
pub use thirty_two::SV_THIRTY_TWO;

extern "C" {
    fn asm_get_satp() -> usize;
    fn asm_set_satp(_: usize);
}

/// Global that stores the type of the page table in use.
/// Provided so software can support multiple types of page tables
/// and pick between them depending on hardware support at runtime.
static mut PAGE_TABLE_TYPE: TableTypes = TableTypes::Sv39;

/// Abstraction over any MMU-backed page table type
#[repr(transparent)]
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

#[derive(Clone, Copy)]
pub struct EntryFlags {
    permissions: PermFlags,
    software: SoftFlags,
    user: bool,
    global: bool,
}

/// Permissions flags in a page table entry.
#[derive(Clone, Copy)]
pub enum PermFlags {
    Leaf = 0b000,
    ReadOnly = 0b001,
    ReadWrite = 0b011,
    ReadWriteExecute = 0b111,
    ReadExecute = 0b101,
}

/// Placeholder for the 2 software-defined bits allowed in the mmu's page table entries
#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct SoftFlags(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PageSize {
    Page,
    Megapage,
    GigaPage,
}

pub struct PageTableDescriptor {
    /// the size of the page table, in bytes (always 4096)
    size: usize,
    /// the number of levels of page tables
    levels: usize,
    /// the size of an entry, in bytes
    entry_size: usize,
    /// description of the "virtual page number" field of virtual addresses
    virtual_segments: &'static [BitGroup],
    /// description of the "physical page number" field of page table entries
    page_segments: &'static [BitGroup],
    /// description of the "physical page number" field of physical addresses
    physical_segments: &'static [BitGroup],
}

/// a Newtype of a tuple (usize, usize) which are (size, offset) where size is #
/// of bits and offset is the bit address of the lowest bit in the group.
type BitGroup = (usize, usize);

fn traverse_root(
    table: &PageTable,
    virtual_address: usize,
    descriptor: &PageTableDescriptor,
) -> usize {
    let level = descriptor.levels - 1;
    traverse(table, virtual_address, level, descriptor)
}

/// Convert a vertual address to a physcial address per the algorithm
/// documented in 4.3.2 of the riscv priviliged isa spec.
fn traverse(
    table: &PageTable,
    virtual_address: usize,
    level: usize,
    descriptor: &PageTableDescriptor,
) -> usize {
    // decompose page table descriptor
    let (pte_size, vpn_segments, ppn_segments, pa_segments, levels) = {
        let d = descriptor;
        (
            d.entry_size,
            d.virtual_segments,
            d.page_segments,
            d.page_segments,
            d.levels,
        )
    };

    // 1) let a be satp.ppn * PAGESIZE. we are disregarding this and using the provided address as the table to search.
    let a = table as *const _ as usize;
    // and let i be LEVELS - 1, again we are disregarding this subtraction  and taking it as input
    let i = level;

    // 2) let pte be the value of the page table entry at address a + va.vpn[i]*PTESIZE
    let va_vpni = extract_bits(virtual_address, &vpn_segments[level]);
    let pte: &usize = unsafe { ((a + va_vpni * pte_size) as *const usize).as_ref().unwrap() };
    // 3) if page table valid bit not set, or if read/write bits set inconsistently, stop
    if !is_valid(pte) || is_invalid(pte) {
        panic!("invalid page table entry");
    }
    // 4) now we know the entry is valid, check if it is readable or executable. if not, it is a branch
    // if it is a leaf, proceed to step 5, otherwise decrement i, checking that i wasn't 0 first,
    // and continue from step 2 after setting a to the next page table based on this pte
    if is_readable(pte) || is_executable(pte) {
        // 5) pte is a leaf.
        // spec describes checking if the memory access is allowed, but that is for the hardware implementation
        // we will just return the address
        // 6) if i > 0, and the appropriate low bits in the pte are not zeroed, this is misaligned
        if extract_bits(*pte, &ppn_segments[level]) != 0 {
            panic!("invalid page table entry");
        }
        // 7) this step manages the access and dirty bits in the pte, which is again only relevent to the hardware implementation
        // 8) ready to form physical address (pa)
        // pa.pgoff = va.pgoff
        let mut pa = virtual_address & ((1 << 12) - 1);
        // if i > 0, this is a super page and the low bits of pa.ppn come from the vpn (e.g. the bits in sections i-1 thru 0)
        if i > 0 {
            for j in 0..i {
                put_bits(virtual_address, &mut pa, &vpn_segments[j], &pa_segments[j]);
            }
        }
        // the highest bits of pa.ppn come from the pte (e.g. the bits in sections LEVELS-1 thru i)
        for k in i..levels - 1 {
            put_bits(*pte, &mut pa, &ppn_segments[k], &pa_segments[k]);
        }
        pa
    } else {
        // pte is a branch, descend to next level
        if level == 0 {
            panic!("invalid page table entry");
        }
        let ppn_descriptor: BitGroup = {
            let mut size = 0;
            for group in ppn_segments {
                let (gsize, _) = group;
                size += gsize;
            }
            (size, ppn_segments[0].1)
        };
        let next_table = extract_bits(*pte, &ppn_descriptor) >> 12;
        let next_table = unsafe { (next_table as *const PageTable).as_ref().unwrap() };
        traverse(next_table, virtual_address, level - 1, descriptor)
    }
}

/// takes the desired bits out of the address, based on a address "segment" descriptors
/// this allows one function to be used to retrieve the virtual address's virtual page number
/// "vpn" for a given level for any type of page table. See riscv priviliged spec,
/// Figure 4.13, Figure 4.16, and Figure 4.19.
fn extract_bits(address: usize, segment: &(usize, usize)) -> usize {
    let (bit_width, offset) = segment;
    let mask = (1 << bit_width) - 1;
    (address >> offset) & mask
}

/// takes bits described in from_segment from "from" and writes them to "to" according to descriptor "to_segment"
fn put_bits(
    from: usize,
    to: &mut usize,
    from_segment: &(usize, usize),
    to_segment: &(usize, usize),
) {
    let mut bits = extract_bits(from, from_segment);
    let (bit_width, offset) = to_segment;
    let mask = ((1 << bit_width) - 1) << offset;
    bits = (bits << offset) & mask;
    *to = *to & !mask;
    *to = *to | bits;
}

/// check lowest bit is set
fn is_valid(entry: &usize) -> bool {
    *entry & 0b1 == 1
}

/// checks read & write bits not inconsistant
fn is_invalid(entry: &usize) -> bool {
    *entry & 0b10 == 0 && *entry & 0b100 == 0b100
}

/// checks bit 1 is set
fn is_readable(entry: &usize) -> bool {
    *entry & 0b10 == 0b10
}

/// checks bit 3 is set
fn is_executable(entry: &usize) -> bool {
    *entry & 0b1000 == 0b1000
}

pub fn translate_address(page_table: &PageTable, virtual_address: usize) -> usize {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => virtual_address,
            Sv32 => traverse_root(page_table, virtual_address, &SV_THIRTY_TWO),
            Sv39 => traverse_root(page_table, virtual_address, &SV_THIRTY_NINE),
            Sv48 => unimplemented!(),
        }
    }
}

pub fn setup() {
    let layout = StaticLayout::get();
    let kernel_page_table = kmem::get_page_table();
    if set_translation_type(TableTypes::Sv39, kernel_page_table) == false {
        panic!("address translation not supported on this processor.");
    }
}

/// Attempts to set the preferred translation table type
/// falling back if unsupported. will fall back to no
/// translation if none is supported by processor.
fn set_translation_type(mode: TableTypes, address: &mut PageTable) -> bool {
    let mode = mode as u8;
    let address = { address as *mut _ } as usize;
    let desired = Satp::from(address, mode);

    let satp = unsafe {
        asm_set_satp(desired.raw());
        asm_get_satp()
    };
    let found = Satp::from_raw(satp);

    if found.raw() == desired.raw() {
        true
    } else {
        false
    }
}
