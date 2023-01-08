use core::cmp::Ordering;

use crate::cpu_status::Satp;
use crate::kmem;
use crate::mem::page::{align_power, dealloc, zalloc};
use crate::mem::{PAGE_ADDR_MASK, PAGE_SIZE};
use crate::{print, println};

mod entry;

mod page_table;

use page_table::forty_eight::SV_FORTY_EIGHT;
use page_table::thirty_nine::SV_THIRTY_NINE;
use page_table::thirty_two::SV_THIRTY_TWO;

pub use entry::EntryFlags;
use entry::PTEntryRead;
use entry::*;

use self::page_table::descriptor::PageTableDescriptor;

extern "C" {
    fn asm_get_satp() -> usize;
    fn asm_set_satp(_: usize);
}

/// Global that stores the type of the page table in use.
/// Provided so software can support multiple types of page tables
/// and pick between them depending on hardware support at runtime.
static mut PAGE_TABLE_TYPE: TableTypes = TableTypes::Sv39;

unsafe fn get_global_descriptor() -> &'static PageTableDescriptor {
    match unsafe { PAGE_TABLE_TYPE} {
        TableTypes::None => panic!("MMU not configured"),
        TableTypes::Sv32 => &SV_THIRTY_TWO,
        TableTypes::Sv39 => &SV_THIRTY_NINE,
        TableTypes::Sv48 => &SV_FORTY_EIGHT,
    }
}

/// Abstraction over any MMU-backed page table type
#[repr(transparent)]
pub struct PageTableUntyped{
    inner: [u8; PAGE_SIZE]
}

impl PageTableUntyped {
    pub fn entry(&self, index: usize, size: usize) -> &PageTableEntryUntyped {
        // ((&mut (self.0).0[index * size]) as *mut _) as *mut usize
        let address = (self as *const PageTableUntyped) as usize + (index * size);
        unsafe { (address as *const PageTableEntryUntyped).as_ref().unwrap() }
        // unsafe { GenericPageTableEntry::at_address(address) }
    }
    pub fn entry_mut(&mut self, index: usize, size: usize) -> &mut PageTableEntryUntyped {
        let address = (self as *mut PageTableUntyped) as usize + (index * size);
        // unsafe { GenericPageTableEntry::at_address_mut(address) }
        unsafe { (address as *mut PageTableEntryUntyped).as_mut().unwrap() }
    }
    pub fn map(&mut self, virt: usize, phys: usize, size: usize, flags: EntryFlags) {
        let descriptor = unsafe {get_global_descriptor()};
        let aligned_vstart = virt & !PAGE_ADDR_MASK;
        let aligned_pstart = phys & !PAGE_ADDR_MASK;
        let page_count = ((align_power(aligned_vstart+size, 12) - aligned_vstart) / PAGE_SIZE).max(1);
        for i in 0..page_count {
            // todo: accomodate larger / varying page sizes
            let v_address = aligned_vstart + (i << 12);
            let p_address = aligned_pstart  + (i << 12);
            let newpages = map_root(self, v_address, p_address, flags, PageSize::Page, descriptor);
            for page in newpages.iter() {
                if *page != 0 {
                    internal_map_range(self, *page, *page, EntryFlags::READ_WRITE, descriptor);
                }
            }
        }
        todo!()
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

fn traverse_root(
    table: &PageTableUntyped,
    virtual_address: usize,
    descriptor: &PageTableDescriptor,
) -> usize {
    let level = descriptor.levels - 1;
    traverse(table, virtual_address, level, descriptor)
}

/// Convert a virtual address to a physical address per the algorithm
/// documented in 4.3.2 of the riscv priviliged isa spec.
fn traverse(
    table: &PageTableUntyped,
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
    // and let i be LEVELS - 1, again we are disregarding this subtraction and taking it as input
    let i = level;

    // 2) let pte be the value of the page table entry at address a + va.vpn[i]*PTESIZE
    let va_vpni = extract_bits(virtual_address, &vpn_segments[level]);
    let pte: (&PageTableEntryUntyped, &PageTableDescriptor) = unsafe {
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
        // ((a + entry_offset) as *const usize).as_ref().unwrap()
        // GenericPageTableEntry::at_address(a + entry_offset)
        let pteu = unsafe { ((a + entry_offset) as *const PageTableEntryUntyped).as_ref().unwrap() };
        (pteu, &descriptor)
    };
    // 3) if page table valid bit not set, or if read/write bits set inconsistently, stop
    if !pte.extract_flags().is_valid() || pte.extract_flags().is_invalid() {
        panic!("invalid page table entry");
    }
    // 4) now we know the entry is valid, check if it is readable or executable. if not, it is a branch
    // if it is a leaf, proceed to step 5, otherwise decrement i, checking that i wasn't 0 first,
    // and continue from step 2 after setting a to the next page table based on this pte
    if pte.extract_flags().is_readable() || pte.extract_flags().is_executable() {
        // 5) pte is a leaf.
        // spec describes checking if the memory access is allowed, but that is for the hardware implementation
        // we will just return the address
        // 6) if i > 0, and the appropriate low bits in the pte are not zeroed, this is misaligned
        if pte.extract_segment(level) != 0 { // if extract_bits(pte.raw(), &ppn_segments[level]) != 0 {}
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
            pa |= pte.extract_segment(k) as usize;
            // put_bits(pte.raw(), &mut pa, &ppn_segments[k], &pa_segments[k]);
        }
        pa
    } else {
        // pte is a branch, descend to next level
        if level == 0 {
            panic!("invalid page table entry");
        }
        // combine all ppn segments from the page table entry descriptor
        // let ppn_descriptor: BitGroup = collapse_descriptor(ppn_segments);

        // let next_table = extract_bits(pte.raw(), &ppn_descriptor) << 12;
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
            (pte.address() as *const PageTableUntyped).as_ref().unwrap()
            // pte.child_table(descriptor)
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
    // println!("extracting {} bits at offset {} from {:x}", bit_width, offset, address);
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

pub fn map_address(
    root: &mut PageTableUntyped,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => [0; 4], //todo: remove
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
        };
    }
}

fn map_root(
    table: &mut PageTableUntyped,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    descriptor: &PageTableDescriptor,
) -> [usize; 4] {
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
    table: &mut PageTableUntyped,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    level: usize,
    descriptor: &PageTableDescriptor,
) -> [usize; 4] {
    let mut newly_allocated_pages: [usize; 4] = [0; 4];
    // println!("mapping {:x} -> {:x} @ {}", virtual_address, physical_address, level);
    let vpn = extract_bits(virtual_address, &descriptor.virtual_segments[level]);
    // let ppn = extract_bits(physical_address, &descriptor.physical_segments[level]);
    // println!("entry index (vpn segment at {}): {:x}", level, vpn);
    let mut entry = (table.entry_mut(vpn, descriptor.entry_size), descriptor);
    // let entry = unsafe { entry.as_mut().unwrap() };

    match page_size.to_level().cmp(&level) {
        Ordering::Equal => {
            // println!("we have reached deepest level needed for this page table size, ready to write entry");
            if entry.extract_flags().is_valid() {
                // println!("writing physical address {:x} to virtual address {:x}, entry is already occupied with physical address {:x}", physical_address, virtual_address, entry.get_address(descriptor));
                if physical_address != entry.address() as usize {
                    panic!("attempted to overwrite existing mmu page table entry");
                }
                // panic!("attempt to overwrite page table entry");
            }
            // when we reach this point, we are ready to write the leaf entry
            // println!("phy: {:x}", physical_address);
            entry.write_address(physical_address as u64);
            entry.write_flags(flags);
            // entry.set_with(physical_address, flags, descriptor);
            // println!("wrote entry {:x}", entry.raw());
        }
        Ordering::Greater => {
            // we should never be able to reach here, sanity check
            panic!("shouldn't be here");
        }
        Ordering::Less => {
            if level == 0 {
                // this check should never fail, todo: check if avoidable
                panic!("Invalid map attempt");
            }

            if !entry.extract_flags().is_valid() {
                // println!("we reached an empty entry, allocating a page for it");
                // check if this entry is valid
                // if not, zalloc a page to store the next page table
                // set this page table's entry value to the address of that table
                // and recurse into that table
                let new_page = zalloc(1).unwrap();
                // println!("z/{}: {:x}", level, new_page as usize);
                let mut branch_flags = flags;
                branch_flags.set_branch();
                entry.write_address(new_page as *mut Page as u64);
                entry.write_flags(branch_flags);
                // entry.set_with(new_page as *mut Page as usize, branch_flags, descriptor);
                let next_table = unsafe { (new_page as *mut PageTableUntyped).as_mut().unwrap() };
                // let next_table = unsafe { entry.child_table_mut(descriptor) };
                // println!("put address {:x} in entry: {:x}", new_page as usize, entry.raw());
                newly_allocated_pages = map(
                    next_table,
                    virtual_address,
                    physical_address,
                    flags,
                    page_size,
                    level - 1,
                    descriptor,
                );
                // println!("  lz/{}: {:x}", level, new_page as usize);
                newly_allocated_pages[level] = new_page as *mut Page as usize;
            } else {
                // println!("we reached an existing entry, getting the address and recursing into it");
                // this entry is valid, extract the next page table address from it and recurse
                // println!("we are at level {}", level);
                // let short_page = extract_bits(entry.raw(), &descriptor.page_segments[level]);
                let page = entry.address();
                // let page = short_page << 12;
                // println!("according to {:x}, next page is at address {:x}", entry.raw(), page);
                let next_table = unsafe { (page as *mut PageTableUntyped).as_mut().unwrap() };
                newly_allocated_pages = map(
                    next_table,
                    virtual_address,
                    physical_address,
                    flags,
                    page_size,
                    level - 1,
                    descriptor,
                );
            }
        }
    };
    newly_allocated_pages
}

pub fn unmap_subtables(table: &mut PageTableUntyped) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => (),
            Sv32 => unmap_root(table, &SV_THIRTY_TWO),
            Sv39 => unmap_root(table, &SV_THIRTY_NINE),
            Sv48 => unmap_root(table, &SV_FORTY_EIGHT),
        }
    }
}

fn unmap_root(table: &mut PageTableUntyped, descriptor: &PageTableDescriptor) {
    unmap(table, descriptor, descriptor.levels - 1)
}

fn unmap(table: &mut PageTableUntyped, descriptor: &PageTableDescriptor, level: usize) {
    for index in 0..descriptor.size {
        let mut entry = (table.entry_mut(index, descriptor.entry_size), descriptor);
        // let entry = unsafe {
        //     (((&mut (table.0).0 as *mut [u8; 4096]) as *mut usize)
        //         .add(index * descriptor.entry_size))
        //     .as_mut()
        //     .unwrap()
        // };
        if entry.extract_flags().is_branch() {
            if level != 0 {
                let page = entry.address() as usize;
                // let page = extract_bits(entry.raw(), &descriptor.page_segments[level]) << 12;
                let next_table = unsafe { (page as *mut PageTableUntyped).as_mut().unwrap() };
                unmap(next_table, descriptor, level - 1);
            } else {
                panic!("invalid page entry encountered");
            }
        }
        entry.invalidate();
    }
    if level != descriptor.levels - 1 {
        dealloc((table as *mut PageTableUntyped) as *mut Page);
    }
}

pub fn translate_address(page_table: &PageTableUntyped, virtual_address: usize) -> usize {
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

pub fn identity_map_range(root: &mut PageTableUntyped, start: usize, end: usize, flags: EntryFlags) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => (),
            Sv32 => internal_map_range(root, start, end, flags, &SV_THIRTY_TWO),
            Sv39 => internal_map_range(root, start, end, flags, &SV_THIRTY_NINE),
            Sv48 => internal_map_range(root, start, end, flags, &SV_FORTY_EIGHT),
        }
    }
}

fn internal_map_range(
    root: &mut PageTableUntyped,
    start: usize,
    end: usize,
    flags: EntryFlags,
    descriptor: &PageTableDescriptor,
) {
    // println!("mapping {:x} to {:x} at page table located {:x}", start, end, ((root as *mut PageTable) as usize));

    // round down start address to page boundary
    let aligned = start & !PAGE_ADDR_MASK;
    let page_count = ((align_power(end, 12) - aligned) / PAGE_SIZE).max(1);
    // println!("becomes {:x} -> {:x}", aligned, aligned + (page_count<<12));
    // println!("mapping {} pages", page_count);
    for i in 0..page_count {
        let address = aligned + (i << 12);
        // println!("mapping page# {} at {:x}", i, address);
        let newpages = map_root(root, address, address, flags, PageSize::Page, descriptor);
        for page in newpages.iter() {
            if *page != 0 {
                // println!("  added a kernel page table: {:x}", *page);
                internal_map_range(root, *page, *page, EntryFlags::READ_WRITE, descriptor);
            }
        }
    }
}

pub fn print_map(table: &PageTableUntyped) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => (),
            Sv32 => inner_print_map(table, &SV_THIRTY_TWO, 0, 0),
            Sv39 => inner_print_map(table, &SV_THIRTY_NINE, 0, 0),
            Sv48 => inner_print_map(table, &SV_FORTY_EIGHT, 0, 0),
        }
    }
}

fn inner_print_map(
    table: &PageTableUntyped,
    descriptor: &PageTableDescriptor,
    base_address: usize,
    descent: usize,
) {
    let max_bits = descriptor.virtual_address_size();
    let bits_known: usize = descriptor
        .virtual_segments
        .iter()
        .take(descent + 1)
        .map(|(bits, _)| *bits)
        .sum();
    let bits_unknown = max_bits - bits_known;
    let page_size = 1 << bits_unknown;
    println!(
        "Reading pagetable located at 0x{:x}:",
        table as *const PageTableUntyped as usize
    );
    // let page_size = 1 << (12+bits_known);
    // println!("memory region described by each entry is: 0x{:x}-bytes", page_size);

    for index in 0..descriptor.size / descriptor.entry_size {
        let resulting_address = base_address + (index * page_size);
        let entry = (table.entry(index, descriptor.entry_size), descriptor);
        if entry.extract_flags().is_valid() {
            println!(
                "{}-{}: 0x{:x}-0x{:x}: {:?}",
                descent,
                index,
                resulting_address,
                resulting_address + page_size - 1,
                entry.0
            );
            if entry.extract_flags().is_branch() {
                // println!("branching");
                let next = entry.address();
                let next_table = unsafe { (next as *const PageTableUntyped).as_ref().unwrap() };

                inner_print_map(next_table, descriptor, resulting_address, descent + 1);

                // println!("rejoining");
            } else {
                // println!("{}-{}: 0x{:x}-0x{:x}: {:?}", descent, index, resulting_address,resulting_address+page_size-1, entry);
            }
        } else {
            // println!("{}-{}: 0x{:x}-0x{:x}: not mapped.", descent, index, resulting_address, resulting_address+page_size-1);
        }
    }
}

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
