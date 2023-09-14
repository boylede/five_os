use core::cmp::Ordering;

use self::{descriptor::BitGroup, forty_eight::Sv48, thirty_nine::Sv39, thirty_two::Sv32};
use super::{align_power, entry::PTEntry, EntryFlags, PageSize};

pub mod descriptor;
pub mod display;
pub mod forty_eight;
pub mod thirty_nine;
pub mod thirty_two;
// pub mod untyped;

/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
pub const PAGE_ADDR_MAGNITIDE: usize = 12;
/// size of the smallest page allocation
pub const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with low 12 bits set
pub const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

pub trait PageTableKind {
    type Entry: Sized + PTEntry + core::fmt::Debug;
    // required methods
    /// the size of the table (all are 1 page)
    fn size(&self) -> usize;
    /// the number of levels this kind of page table has
    fn depth(&self) -> usize;
    /// the number of bytes per entry
    fn entry_size(&self) -> usize;
    /// a description of the entry's address segments
    fn entry_segments(&self) -> &[BitGroup];
    /// a description of te physical address's segments
    fn physical_segments(&self) -> &[BitGroup];
    /// a description of the virtual address's segments
    fn virtual_segments(&self) -> &[BitGroup];
    // provided methods
    /// number of bits in the virtual address
    fn virtual_address_size(&self) -> usize {
        self.virtual_segments()
            .iter()
            .map(|(n, _)| n)
            .sum::<usize>()
            + 12
    }
}

pub enum AnyPageTable {
    Off,
    Sv32(PageTable<Sv32>),
    Sv39(PageTable<Sv39>),
    Sv48(PageTable<Sv48>),
}

#[derive(Debug)]
pub struct PageTable<K>
where
    K: PageTableKind,
{
    pub(crate) table: *const K::Entry,
    pub(crate) kind: K,
}

impl<K> PageTable<K>
where
    K: PageTableKind + core::fmt::Debug + Copy,
{
    pub fn in_place(table: *const u8, kind: K) -> PageTable<K> {
        let table = table as *const K::Entry;
        PageTable { table, kind }
    }
    /// Get the entry at this index
    ///
    /// ## Safety
    /// todo: currently no safety checks in place. FIXME
    pub fn entry(&self, index: usize) -> &<K as PageTableKind>::Entry {
        assert!(
            index < self.kind.size() / self.kind.entry_size(),
            "indexed out of array"
        );
        // let mut uart = unsafe { Uart0::new() };
        // println!(
        //     uart,
        //     "page table item located at {:x}", self as *const _ as usize
        // );
        // println!(uart, "{:?}", self);
        // println!(
        //     uart,
        //     "adding {:x} to {:x} results in {:x}",
        //     index,
        //     self.table as *const _ as usize,
        //     self.table.wrapping_add(index) as *const _ as usize
        // );
        unsafe { self.table.wrapping_add(index).as_ref().unwrap() }
    }

    /// Map thee given physical addresses to corresponding virtual addresses
    ///
    /// ## Safety
    /// todo: currently no safety checks in place
    pub fn identity_map(
        &self,
        start: usize,
        end: usize,
        flags: EntryFlags,
        zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
    ) {
        // let mut uart = unsafe { Uart0::new() };
        // println!(
        //     uart,
        //     "identity mapping from {:x} to {:x}, in table at {:x}",
        //     start,
        //     end,
        //     self as *const _ as usize
        // );
        internal_map_range(self, start, end, flags, zalloc)
    }
}

fn internal_map_range<K>(
    root: &PageTable<K>,
    start: usize,
    end: usize,
    flags: EntryFlags,
    zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
) where
    K: PageTableKind + core::fmt::Debug + Copy,
{
    // let mut uart = unsafe { Uart0::new() };
    // println!(
    //     uart,
    //     "mapping {:x} to {:x} at page table located {:x}",
    //     start,
    //     end,
    //     ((root as *const _) as usize)
    // );

    // round down start address to page boundary
    let aligned = start & !PAGE_ADDR_MASK;
    let page_count = ((align_power(end, 12) - aligned) / PAGE_SIZE).max(1);
    // println!(
    //     uart,
    //     "becomes {:x} -> {:x}",
    //     aligned,
    //     aligned + (page_count << 12)
    // );
    // println!(uart, "mapping {} pages", page_count);
    for i in 0..page_count {
        let address = aligned + (i << 12);
        // println!(uart, "mapping page# {} at {:x}", i, address);
        // println!(uart, "imr:{:?}", root);
        let newpages = map_root(root, address, address, flags, PageSize::Page, zalloc);
        for page in newpages.iter() {
            if *page != 0 {
                // println!(uart, "  added a kernel page table: {:x}", *page);
                internal_map_range(root, *page, *page, EntryFlags::READ_WRITE, zalloc);
            }
        }
    }
}

fn map_root<K>(
    table: &PageTable<K>,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
) -> [usize; 4]
where
    K: PageTableKind + core::fmt::Debug + Copy,
{
    map(
        table,
        virtual_address,
        physical_address,
        flags,
        page_size,
        table.kind.depth() - 1,
        zalloc,
    )
}

fn map<K>(
    table: &PageTable<K>,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    level: usize,
    zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
) -> [usize; 4]
where
    K: PageTableKind + core::fmt::Debug + Copy,
{
    // let mut uart = unsafe { Uart0::new() };
    let mut newly_allocated_pages: [usize; 4] = [0; 4];
    // println!(uart, "map:{:?}", table);
    // println!(
    //     uart,
    //     "mapping {:x} -> {:x} @ {}", virtual_address, physical_address, level
    // );
    let vpn = extract_bits(virtual_address, &table.kind.virtual_segments()[level]);

    // let ppn = extract_bits(physical_address, &descriptor.physical_segments[level]);
    // println!(uart, "entry index (vpn segment at {}): {:x}", level, vpn);
    let entry = table.entry(vpn);
    // let entry = unsafe { entry.as_mut().unwrap() };

    // println!(
    //     uart,
    //     "got mutable entry located at {:x}, from table located at {:x}",
    //     entry as *const _ as usize,
    //     table as *const _ as usize
    // );
    let old_value = entry.load();
    match page_size.to_level().cmp(&level) {
        Ordering::Equal => {
            // println!(uart, "we have reached deepest level needed for this page table size, ready to write entry");
            if entry.read_flags().is_valid() {
                // println!(uart, "writing physical address {:x} to virtual address {:x}, entry is already occupied with physical address (TODO)", physical_address, virtual_address);
                if physical_address != entry.read_address() as usize {
                    panic!("attempted to overwrite existing mmu page table entry");
                }
                // panic!("attempt to overwrite page table entry");
            }
            // when we reach this point, we are ready to write the leaf entry
            // println!(uart, "phy: {:x}", physical_address);
            if !entry.write(old_value, physical_address as u64, flags) {
                panic!("Failed to write to page table - concurrent access?");
            }

            // entry.set_with(physical_address, flags, descriptor);
            // println!(uart, "wrote entry {:?}", entry);
        }
        Ordering::Greater => {
            // we should never be able to reach here, sanity check
            // println!(uart, "greater");
            panic!("shouldn't be here");
        }
        Ordering::Less => {
            if level == 0 {
                // this check should never fail, todo: check if avoidable
                // println!(uart, "failless");
                panic!("Invalid map attempt");
            }
            // println!(uart, "less");
            if !entry.read_flags().is_valid() {
                // println!(uart, "we reached an empty entry, allocating a page for it");
                // check if this entry is valid
                // if not, zalloc a page to store the next page table
                // set this page table's entry value to the address of that table
                // and recurse into that table
                let new_page = zalloc(1).unwrap();
                // println!(uart, "z/{}: {:x}", level, new_page as usize);
                let mut branch_flags = flags;
                branch_flags.set_branch();
                if !entry.write(old_value, new_page as *mut _ as u64, branch_flags) {
                    panic!("failed to write to page table: concurrent access?");
                }
                // entry.set_with(new_page as *mut Page as usize, branch_flags, descriptor);
                let next_table = PageTable::in_place(new_page, table.kind);
                // let next_table = unsafe { entry.child_table_mut(descriptor) };
                // println!(
                //     uart,
                //     "put address {:x} in entry: {:?}", new_page as usize, entry
                // );
                newly_allocated_pages = map(
                    &next_table,
                    virtual_address,
                    physical_address,
                    flags,
                    page_size,
                    level - 1,
                    zalloc,
                );
                // println!(uart, "  lz/{}: {:x}", level, new_page as usize);
                newly_allocated_pages[level] = new_page as *mut _ as usize;
            } else {
                // println!(
                //     uart,
                //     "we reached an existing entry, getting the address and recursing into it"
                // );
                // this entry is valid, extract the next page table address from it and recurse
                // println!(uart, "we are at level {}", level);
                // let short_page = extract_bits(entry.raw(), &descriptor.page_segments[level]);
                let page = entry.read_address();
                // let page = short_page << 12;
                // println!(
                //     uart,
                //     "according to {:?}, next page is at address {:x}", entry, page
                // );
                let next_table = PageTable::in_place(page as *const u8, table.kind);

                newly_allocated_pages = map(
                    &next_table,
                    virtual_address,
                    physical_address,
                    flags,
                    page_size,
                    level - 1,
                    zalloc,
                );
            }
        }
    };
    newly_allocated_pages
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
