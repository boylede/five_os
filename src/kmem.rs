use crate::mmu::PageTable;
use crate::page::{align_power, zalloc, PAGE_SIZE};

/// Allocates memory for the kernel.
use core::{mem::size_of, ptr::null_mut};

/// Number of pages allocated for kernel use
static mut KMEM_ALLOC: usize = 0;
/// pointer to first byte of kernel allocation
static mut KMEM_HEAD: *mut AllocList = null_mut();
/// MMU page table for kernel
static mut KMEM_PAGE_TABLE: *mut PageTable = null_mut();

/// Safe wrapper around page table global
pub fn get_page_table() -> &'static mut PageTable {
    unsafe {
    // SAFETY: we are converting a mutable pointer to a mutable reference,
    // we need to ensure that the pointer is null, or all of the following
    // are true of the memory location & range:
    // a) initialized, b) valid for PageTable, c) properly aligned for
    // PageTable, d) non-null, e) contain a single allocated object f) no
    // other access to this location occurs during the lifetime
    // of the mutable reference we are creating.
    // a: page table was zero'd when allocated
    // b: global symbol is the correct size, alignment for pagetable as the pointer was declared with this same type
    // c: same
    // d: non-null per a
    // e: does not overlap with any other object, as we ensure through our page-grained allocation
    // f: we will ensure single access in the future
        KMEM_PAGE_TABLE.as_mut().unwrap()
    }
}
/// safe wrapper around static mut
pub fn get_heap_location() -> usize {
    unsafe { KMEM_HEAD as usize }
}

pub fn allocation_count() -> usize {
    unsafe { KMEM_ALLOC as usize }
}

/// An AllocList stores the size and status of the following sequence of bytes
/// another AllocList can be expected at alloc_list.add(size) bytes later;
/// these will be placed in allocated pages to subdvide them into memory regions
struct AllocList {
    flags_size: usize,
}

#[cfg(target_pointer_width = "64")]
const TAKEN_BIT: usize = 1 << 63;
#[cfg(target_pointer_width = "32")]
const TAKEN_BIT: usize = 1 << 31;

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
        // ensure taken_bit is clear in input
        assert!(size & TAKEN_BIT == 0);
        let taken = self.flags_size & TAKEN_BIT;
        self.flags_size = (size & !TAKEN_BIT) | taken;
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
        // SAFETY: We are writing to static globals, which requires that
        // we ensure exclusive access. currently, we only write these
        // items once at startup and then they are immutable. in the
        // future, we can add some protection.
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

pub fn kzmalloc(_size: usize) -> *mut u8 {
    unimplemented!()
}

pub fn kmalloc(size: usize) -> *mut u8 {
    // scale the size to 8-byte boundaries (lowest three bits zero)
    // and add space required to store metadata
    let size = align_power(size, 3) + size_of::<AllocList>();

    // local variable will be used to walk through the kernel memory space
    // one allocation at a time
    let mut head = unsafe { 
        // SAFETY: access to static global, 
        // we must ensure no one has mutable access to head
        // currently, we treat all the KMEM_ globals as only mutable during setup.
        KMEM_HEAD 
    };
    let mut current_allocation = unsafe {
        // SAFETY: 
        head.as_mut()
    }.unwrap();
    // local variable to compare to head while walking kernel memory
    let tail = unsafe { (head as *mut u8).add(KMEM_ALLOC * PAGE_SIZE) } as *mut AllocList;
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
                let next = unsafe {
                    ((head as *mut u8).add(size) as *mut AllocList)
                        .as_mut()
                        .unwrap()
                };
                next.clear();
                next.set_free();
                next.set_size(remainder);
                current_allocation.set_size(size);
            } else {
                // take everything
                current_allocation.set_size(chunk_size);
            }
            // offset pointer by size of the metadata and coerce to general pointer
            return unsafe { head.add(1) } as *mut u8;
        } else {
            // go to next chunk
            head =
                unsafe { (head as *mut u8).add(current_allocation.get_size()) } as *mut AllocList;
            current_allocation = unsafe { head.as_mut() }.unwrap();
        }
    }
    // failed to allocate any memory, return null pointer
    null_mut()
}
