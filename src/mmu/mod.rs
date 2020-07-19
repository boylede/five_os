use core::cmp::Ordering;

use crate::cpu_status::Satp;
use crate::kmem;
use crate::page::{zalloc, PAGE_SIZE, dealloc};


mod forty_eight;
mod thirty_nine;
mod thirty_two;

use forty_eight::SV_FORTY_EIGHT;
use thirty_nine::SV_THIRTY_NINE;
use thirty_two::SV_THIRTY_TWO;

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

impl PageTable {
    pub fn entry(&mut self, index: usize, size: usize) -> *mut usize {
        ((&mut (self.0).0[index * size]) as *mut _) as *mut usize
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

pub struct Page([u8; PAGE_SIZE]);

#[derive(Clone, Copy)]
pub struct EntryFlags {
    permissions: PermFlags,
    software: SoftFlags,
    user: bool,
    global: bool,
}

impl EntryFlags {
    /// Puts each bitflag into the lower 9 bits of a usize,
    /// ready for insertion into any type of page table entry
    /// also sets valid flag
    pub fn to_entry(&self) -> usize {
        0b1 | // set valid bit
        (self.permissions as usize) << 1 |
        (self.user as usize) << 4 | 
        (self.global as usize) << 5
    }
    pub fn software(&mut self) -> &mut SoftFlags {
        &mut self.software
    }
}

/// Permissions flags in a page table entry.
#[derive(Clone, Copy)]
enum PermFlags {
    Leaf = 0b000,
    ReadOnly = 0b001,
    ReadWrite = 0b011,
    ReadWriteExecute = 0b111,
    ReadExecute = 0b101,
}

/// Placeholder for the 2 software-defined bits allowed in the mmu's page table entries
#[derive(Clone, Copy, Default)]
pub struct SoftFlags(u8);

impl SoftFlags {
    fn set(&mut self, value: u8) {
        self.0 = value & 0b11;
    }
    fn get(&self) -> u8 {
        self.0
    }
    fn clear(&mut self) {
        self.0 = 0
    }
    fn set_a(&mut self) {
        self.0 |= 0b01
    }
    fn set_b(&mut self) {
        self.0 |= 0b10
    }
    fn get_a(&mut self) -> bool {
        self.0 & 0b01 == 0b01
    }
    fn get_b(&mut self) -> bool{
        self.0 & 0b10 == 0b10
    }
}

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

struct PageTableDescriptor {
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

fn collapse_descriptor(segments: &[BitGroup]) -> BitGroup {
    let mut size = 0;
    for group in segments {
        let (gsize, _) = group;
        size += gsize;
    }
    (size, segments[0].1)
}

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
    let (table_size, pte_size, vpn_segments, ppn_segments, pa_segments, levels) = {
        let d = descriptor;
        (
            d.size,
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
    let pte: &usize = unsafe {
        // SAFETY: we are converting an arbitrary memory address to a usize reference,
        // so we need to be sure that the memory address is a) initialized,
        // b) contents valid for usize, c) aligned for usize, and d) no concurrent
        // access to this address can modify it.
        // a: this page table was allocated with zalloc, so the memory is known zero, or was written since then by 
        // b: all initialized memory is valid for integral types
        // c: the address is aligned for usize because a is aligned for usize,
        // and the offset (va_vpni) is scaled by pte_size which represents the required alignment
        // d: we cannot prove this yet, but we are single-threaded at the moment so
        // when we switch to multi-threaded we will need to protect page tables with a mutex, semaphore or simular structure
        
        let entry_offset = va_vpni * pte_size;
        // check that va_vpni does not push us past the end of the table
        // this check *should* be redundant because the page table descriptor "vpn_segments"
        // passed to extract_bits should ensure only values of a limited magnitude can be
        // returned from that function, but we will check here to be sure
        assert!(entry_offset <= table_size - pte_size);
        ((a + entry_offset) as *const usize).as_ref().unwrap()
    };
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
        // combine all ppn segments from the page table entry descriptor
        let ppn_descriptor: BitGroup = collapse_descriptor(ppn_segments);

        let next_table = extract_bits(*pte, &ppn_descriptor) << 12;
        let next_table = unsafe {
            // SAFETY: we are converting an arbitrary usize to a PageTable reference, so we need
            // to be sure that the memory address is a) initialized, b) contents valid
            // for PageTable, c) aligned for PageTable, and d) no concurrent access to this
            // address can modify it.
            // a: page tables are created with zalloc, so are always initialized
            // b: PageTable is an array of integral types, with total size equal to a memory
            // page, since the page was zero'd and since initialized memory is valid for all integral types, we are valid for PageTable
            // c: we are shifting the output of extract_bits by 12, which ensures that the low 12 bits are zero, as required
            // d: again, this will need to be protected by a mutex or semaphore, once we add support for multiple cores
            (next_table as *const PageTable).as_ref().unwrap()
        };
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
    *to &= !mask;
    *to |= bits;
}

/// check lowest bit is set
fn is_valid(entry: &usize) -> bool {
    *entry & 0b1 == 1
}

/// checks read & write bits not inconsistant
fn is_invalid(entry: &usize) -> bool {
    *entry & 0b10 == 0 && *entry & 0b100 == 0b100
}

fn invalidate(entry: &mut usize) {
    *entry = 0;
}

/// checks bit 1 is set
fn is_readable(entry: &usize) -> bool {
    *entry & 0b10 == 0b10
}

/// checks bit x is set
fn is_writable(entry: &usize) -> bool {
    *entry & 0b100 == 0b100
}


/// checks bit 3 is set
fn is_executable(entry: &usize) -> bool {
    *entry & 0b1000 == 0b1000
}

fn is_branch(entry: &usize) -> bool {
    is_valid(entry) && !is_readable(entry) && !is_executable(entry) && !is_writable(entry)
}

/// produce a page table entry based on the provided descriptor,
/// permissions bits, and software bits, and sets valid bit
fn create_entry(address: usize, flags: EntryFlags, descriptor: &PageTableDescriptor) -> usize {
    // put_bits(address, &mut entry, from_segment: &(usize, usize), &descriptor.page_segments);
    let mut bits = 0;
    for level in 0..descriptor.levels {
        let (bit_width, offset) = descriptor.page_segments[level];
        let mask = ((1 << bit_width) - 1) << offset;
        bits = (address << offset) & mask;
    }
    bits | flags.to_entry()
}

pub fn map_address(
    root: &mut PageTable,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => (),
            Sv32 => map_root(
                root,
                virtual_address,
                physical_address,
                flags,
                page_size,
                &SV_THIRTY_TWO,
            ),
            Sv39 => map_root(
                root,
                virtual_address,
                physical_address,
                flags,
                page_size,
                &SV_THIRTY_NINE,
            ),
            Sv48 => map_root(
                root,
                virtual_address,
                physical_address,
                flags,
                page_size,
                &SV_FORTY_EIGHT,
            ),
        }
    }
}

fn map_root(
    table: &mut PageTable,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    descriptor: &PageTableDescriptor,
) {
    map(
        table,
        virtual_address,
        physical_address,
        flags,
        page_size,
        descriptor.levels - 1,
        descriptor,
    )
}

fn map(
    table: &mut PageTable,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    level: usize,
    descriptor: &PageTableDescriptor,
) {
    let vpn = extract_bits(virtual_address, &descriptor.virtual_segments[level]);
    // let ppn = extract_bits(physical_address, &descriptor.physical_segments[level]);
    
    
    let entry: *mut usize = table.entry(vpn, descriptor.entry_size);
    let entry = unsafe {entry.as_mut().unwrap()};

    match page_size.to_level().cmp(&level) {
        Ordering::Equal => {
            if is_valid(entry) {
                panic!("attempt to overwrite page table entry");
            }
            // when we reach this point, we are ready to write the leaf entry
            let new_entry = create_entry(physical_address, flags, descriptor); //todo: check the pointer is positioned correctly, might want to insert a call to put_bits here
            *entry = new_entry;
        },
        Ordering::Greater => {
            // we should never be able to reach here, sanity check
            panic!("shouldn't be here");
        },
        Ordering::Less => {
            if level == 0 {
                // this check should never fail, todo: check if avoidable
                panic!("Invalid map attempt");
            }
            if !is_valid(entry) {
                // check if this entry is valid
                // if not, zalloc a page to store the next page table
                // set this page table's entry value to the address of that table
                // and recurse into that table
                let new_page = zalloc(1);
                let new_entry = create_entry(new_page as usize, flags, descriptor); //todo: check the pointer is positioned correctly, might want to insert a call to put_bits here
                *entry = new_entry;
                let next_table = unsafe {(new_page as *mut PageTable).as_mut().unwrap()};
                map(next_table, virtual_address, physical_address, flags, page_size, level - 1, descriptor);
            } else {
                // this entry is valid, extract the next page table address from it and recurse
                let page = extract_bits(*entry, &descriptor.page_segments[level]) << 12;
                let next_table = unsafe {(page as *mut PageTable).as_mut().unwrap()};
                map(next_table, virtual_address, physical_address, flags, page_size, level - 1, descriptor);
            }
        },
    };
}

pub fn unmap_subtables(
    table: &mut PageTable,
) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => (),
            Sv32 => unmap_root(
                table,
                &SV_THIRTY_TWO,
            ),
            Sv39 => unmap_root(
                table,
                &SV_THIRTY_NINE,
            ),
            Sv48 => unmap_root(
                table,
                &SV_FORTY_EIGHT,
            ),
        }
    }
}

fn unmap_root(
    table: &mut PageTable,
    descriptor: &PageTableDescriptor,
) {
    unmap(
        table,
        descriptor,
        descriptor.levels - 1,
    )
}

fn unmap(table: &mut PageTable, descriptor: &PageTableDescriptor, level: usize) {
    for index in 0..descriptor.size {
        let entry = unsafe {
            (((&mut (table.0).0 as *mut [u8; 4096]) as *mut usize).add(index * descriptor.entry_size)).as_mut().unwrap()
        };
        if is_branch(entry) {
            if level != 0 {
                let page = extract_bits(*entry, &descriptor.page_segments[level]) << 12;
                let next_table = unsafe {(page as *mut PageTable).as_mut().unwrap()};
                unmap(next_table, descriptor, level - 1);
            } else {
                panic!("invalid page entry encountered");
            }
        }
        invalidate(entry);
    }
    if level != descriptor.levels - 1 {
        dealloc((table as *mut PageTable) as *mut usize);
    }
}

pub fn translate_address(page_table: &PageTable, virtual_address: usize) -> usize {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => virtual_address,
            Sv32 => traverse_root(page_table, virtual_address, &SV_THIRTY_TWO),
            Sv39 => traverse_root(page_table, virtual_address, &SV_THIRTY_NINE),
            Sv48 => traverse_root(page_table, virtual_address, &SV_FORTY_EIGHT),
        }
    }
}

pub fn setup() {
    let kernel_page_table = kmem::get_page_table();
    if !set_translation_type(TableTypes::Sv39, kernel_page_table) {
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

    found.raw() == desired.raw() 
}
