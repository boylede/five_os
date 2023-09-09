extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::null_mut;

/// An AllocList stores the size and status of the following sequence of bytes
/// another AllocList can be expected at alloc_list.add(size) bytes later;
/// these will be placed in allocated pages to subdvide them into memory regions
pub struct AllocList {
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
        self.flags_size & !TAKEN_BIT
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

/// page-sized allocator.... todo: make thread safe
pub struct BumpPointerAlloc<const P: usize> {
    head: usize,
    tail: usize,
}

unsafe impl<const P: usize> GlobalAlloc for BumpPointerAlloc<P> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = align_to(
            layout.size() + size_of::<AllocList>(),
            layout.align().max(8),
        );

        let mut head = self.head as *mut AllocList;
        let mut current_allocation = (head as *mut AllocList).as_mut().unwrap();

        let tail = self.tail as *mut AllocList;
        let mut ret = null_mut() as *mut AllocList;
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
                    let next =
                        unsafe { ((head as usize + size) as *mut AllocList).as_mut().unwrap() };
                    next.clear();
                    next.set_free();
                    next.set_size(remainder);
                    current_allocation.set_size(size);
                } else {
                    // take everything
                    current_allocation.set_size(chunk_size);
                }
                // offset pointer by size of the metadata and coerce to general pointer
                ret = unsafe { head.add(1) } as *mut AllocList;
                break;
            } else {
                // go to next chunk
                head = (head as usize + current_allocation.get_size()) as *mut AllocList;
                current_allocation = unsafe { head.as_mut() }.unwrap();
            }
        }
        ret as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        let address = ptr as usize;
        let mut head = self.head as *mut AllocList;
        let tail = self.tail as *mut AllocList;
        let mut current_allocation = unsafe { head.as_mut().unwrap() };
        while head < tail {
            if current_allocation.is_taken() {
                let current_end = current_allocation.get_size() + head as usize;
                if current_end > address as usize {
                    current_allocation.set_free();
                    break;
                } else {
                    head = (head as usize + current_allocation.get_size()) as *mut AllocList;
                    current_allocation = unsafe { head.as_mut() }.unwrap();
                }
            } else {
                head = (head as usize + current_allocation.get_size()) as *mut AllocList;
                current_allocation = unsafe { head.as_mut() }.unwrap();
            }
        }
        self.coalesce()
    }
}

impl<const P: usize> BumpPointerAlloc<P> {
    pub const fn new(head: usize, tail: usize) -> BumpPointerAlloc<P> {
        BumpPointerAlloc { head, tail }
    }
    pub fn head(&self) -> usize {
        self.head
    }
    pub fn tail(&self) -> usize {
        self.tail
    }
    // todo: rewrite to make thread safe
    pub fn coalesce(&self) {
        unsafe {
            let mut head = self.head as *mut AllocList;
            let tail = self.tail as *mut AllocList;
            while head < tail {
                let next = (head as *mut u8).add((*head).get_size()) as *mut AllocList;
                if (*head).get_size() == 0 {
                    todo!();
                    break;
                } else if next >= tail {
                    todo!();
                    break;
                } else if (*head).is_free() && (*next).is_free() {
                    (*head).set_size((*head).get_size() + (*next).get_size());
                }
                head = (head as *mut u8).add((*head).get_size()) as *mut AllocList;
            }
        }
    }
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}
