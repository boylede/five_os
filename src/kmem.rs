/// Allocates memory for the kernel.
use core::{mem::size_of, ptr::null_mut};
use crate::page::{zalloc, PAGE_SIZE, align_power};
use crate::mmu::PageTable;

/// Number of pages allocated for kernel use
static mut KMEM_ALLOC: usize = 0;
/// pointer to first byte of kernel allocation
static mut KMEM_HEAD: *mut AllocList = null_mut();
/// MMU page table for kernel
static mut KMEM_PAGE_TABLE: *mut PageTable = null_mut();

/// An AllocList stores the size and status of the following sequence of bytes
/// another AllocList can be expected at alloc_list.add(size) bytes later;
/// these will be placed in allocated pages to subdvide them into memory regions
struct AllocList {
    flags_size: u64,
}

const TAKEN_BIT: u64 = (1<<63);

impl AllocList {
    pub fn is_taken(&self) -> bool {
        self.flags_size & TAKEN_BIT != 0
    }
    pub fn set_taken(&mut self) {
        self.flags_size |= TAKEN_BIT
    }
    pub fn get_size(&self) -> usize {
        (self.flags_size & !TAKEN_BIT) as usize
    }
    pub fn set_size(&mut self, size: usize) {
        assert!(size as u64 | TAKEN_BIT == 0);
        let taken = self.flags_size & TAKEN_BIT;
        self.flags_size = (size as u64 & !TAKEN_BIT) | taken;
    }
    pub fn is_free(&self) -> bool {
        !self.is_taken()
    }
    pub fn set_free(&mut self) {
        self.flags_size &= !TAKEN_BIT
    }
    pub fn clear(&mut self) {
        self.flags_size = 0;
    }
}


pub fn setup() {
    unsafe {
        KMEM_ALLOC = 512;
        let k_alloc = zalloc(KMEM_ALLOC);
        assert!(!k_alloc.is_null());
        KMEM_HEAD = k_alloc as *mut AllocList;
        let kmem = KMEM_HEAD.as_mut().unwrap();
        kmem.set_free();
        kmem.set_size(KMEM_ALLOC * PAGE_SIZE);
        KMEM_PAGE_TABLE = zalloc(1) as *mut PageTable;
    }
}

pub fn kzmalloc(size: usize) -> *mut u8 {
    unimplemented!()
}

pub fn kmalloc(size: usize) -> *mut u8 {
    // scale the size to 8-byte boundaries (lowest three bits zero)
    // and add space required to store metadata
    let size = align_power(size, 3) + size_of::<AllocList>();
    
    // local variable will be used to walk through the kernel memory space
    // one allocation at a time
    let mut head = unsafe {KMEM_HEAD};
    let mut current_allocation = unsafe {head.as_mut()}.unwrap();
    // local variable to compare to head while walking kernel memory
    let tail = unsafe {(head as *mut u8).add(KMEM_ALLOC * PAGE_SIZE)} as *mut AllocList;
    while head < tail {
        if current_allocation.is_free() && size <= current_allocation.get_size() {
            // split this chunk and return
            let chunk_size = current_allocation.get_size();
            let remainder = chunk_size - size;
            current_allocation.set_taken();
            // chunks smaller than 8 bytes + metadata can't be allocated anyway
            // so just take the whole chunk in that case
            if remainder >= size_of::<AllocList>() + 8 {
                // split the chunk
                let next = unsafe {((head as *mut u8).add(size) as *mut AllocList).as_mut().unwrap()};
                next.clear();
                next.set_free();
                next.set_size(remainder);
                current_allocation.set_size(size);
            } else {
                // take everything
                current_allocation.set_size(chunk_size);
            }
            // offset pointer by size of the metadata and coerce to general pointer
            return unsafe {head.add(1)} as *mut u8;

        } else {
            // go to next chunk
            head = unsafe {(head as *mut u8).add(current_allocation.get_size())} as *mut AllocList;
            current_allocation = unsafe {head.as_mut()}.unwrap();
        }
        
    }
    // failed to allocate any memory, return null pointer
    null_mut()
}
