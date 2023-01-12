use core::cmp::Ordering;

use crate::mmu::{
    align_power,
    entry::{PTEntryRead, PTEntryWrite},
    EntryFlags, Page, PageSize,
};

use self::entry::PageTableEntryUntyped;

use super::{descriptor::PageTableDescriptor, PAGE_ADDR_MASK, PAGE_SIZE};

pub mod entry;

/// Abstraction over any MMU-backed page table type
#[repr(transparent)]
pub struct PageTableUntyped {
    inner: [u8; PAGE_SIZE],
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
    pub fn map(
        &mut self,
        descriptor: &PageTableDescriptor,
        virt: usize,
        phys: usize,
        size: usize,
        flags: EntryFlags,
        zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
    ) {
        let aligned_vstart = virt & !PAGE_ADDR_MASK;
        let aligned_pstart = phys & !PAGE_ADDR_MASK;
        let page_count =
            ((align_power(aligned_vstart + size, 12) - aligned_vstart) / PAGE_SIZE).max(1);
        for i in 0..page_count {
            // todo: accomodate larger / varying page sizes
            let v_address = aligned_vstart + (i << 12);
            let p_address = aligned_pstart + (i << 12);
            let newpages = map_root(
                self,
                v_address,
                p_address,
                flags,
                PageSize::Page,
                descriptor,
                zalloc,
            );
            for page in newpages.iter() {
                if *page != 0 {
                    internal_map_range(
                        self,
                        *page,
                        *page,
                        EntryFlags::READ_WRITE,
                        descriptor,
                        zalloc,
                    );
                }
            }
        }
        todo!()
    }
    pub fn identity_map(
        &mut self,
        descriptor: &PageTableDescriptor,
        start: usize,
        end: usize,
        flags: EntryFlags,
        zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
    ) {
        unsafe {
            // use TableTypes::*;
            // match PAGE_TABLE_TYPE {
            //     None => (),
            //     Sv32 => internal_map_range(self, start, end, flags, &SV_THIRTY_TWO, zalloc),
            //     Sv39 => internal_map_range(self, start, end, flags, &SV_THIRTY_NINE, zalloc),
            //     Sv48 => internal_map_range(self, start, end, flags, &SV_FORTY_EIGHT, zalloc),
            // }
            internal_map_range(self, start, end, flags, descriptor, zalloc)
        }
    }
}

fn internal_map_range(
    root: &mut PageTableUntyped,
    start: usize,
    end: usize,
    flags: EntryFlags,
    descriptor: &PageTableDescriptor,
    zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
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
        let newpages = map_root(
            root,
            address,
            address,
            flags,
            PageSize::Page,
            descriptor,
            zalloc,
        );
        for page in newpages.iter() {
            if *page != 0 {
                // println!("  added a kernel page table: {:x}", *page);
                internal_map_range(
                    root,
                    *page,
                    *page,
                    EntryFlags::READ_WRITE,
                    descriptor,
                    zalloc,
                );
            }
        }
    }
}

fn map_root(
    table: &mut PageTableUntyped,
    virtual_address: usize,
    physical_address: usize,
    flags: EntryFlags,
    page_size: PageSize,
    descriptor: &PageTableDescriptor,
    zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
) -> [usize; 4] {
    map(
        table,
        virtual_address,
        physical_address,
        flags,
        page_size,
        descriptor.levels - 1,
        descriptor,
        zalloc,
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
    zalloc: &mut dyn FnMut(usize) -> Option<*mut u8>,
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
                    zalloc,
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
