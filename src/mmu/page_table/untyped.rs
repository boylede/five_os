use crate::{mem::{PAGE_SIZE, PAGE_ADDR_MASK, page::align_power}, mmu::{EntryFlags, get_global_descriptor, map_root, internal_map_range, PageSize}};

use self::entry::PageTableEntryUntyped;

pub mod entry;

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