use core::assert;
use core::mem::size_of;

pub mod bitmap;
use bitmap::PageMarker;

pub struct PageContents(core::sync::atomic::AtomicU8);

/// an array of L bytes, intended to represent a page of memory.
pub struct Page<const L: usize>([u8; L]);

/// a page-based allocator, const A is the alignment and size of a page
pub struct PageAllocator<const A: usize> {
    head: usize,
    tail: usize,
}

impl<const A: usize> PageAllocator<A> {
    /// todo: delete this it is a crime
    pub const fn uninitalized() -> PageAllocator<A> {
        PageAllocator { head: 0, tail: 0 }
    }
    /// # Safety
    /// This will access the underlying memory directly and dereference within the given range
    /// ((end - start) / alignment ) - (align(start,alignment)-start) should be greater than 0, e.g. there should be atleast 1 allocatable
    /// page after using the space starting at S as a bitmap indicating which pages are used.
    pub unsafe fn new(head: usize, tail: usize) -> PageAllocator<A> {
        let mut this = PageAllocator { head, tail };
        this.clear_bitmap();
        this
    }
    /// Provides the head and tail for debug purposes
    pub fn info(&self) -> (usize,usize) {
        (self.head, self.tail)
    }
    fn clear_bitmap(&mut self) {
        for entry in self.bitmap_mut().iter_mut() {
            entry.clear()
        }
    }
    pub fn bitmap(&self) -> &[PageMarker] {
        unsafe {
            core::slice::from_raw_parts::<PageMarker>(self.head as *const _, self.page_count())
        }
    }
    fn bitmap_mut(&mut self) -> &mut [PageMarker] {
        unsafe {
            core::slice::from_raw_parts_mut::<PageMarker>(self.head as *mut _, self.page_count())
        }
    }
    /// Allocates the number of pages requested
    pub fn alloc(&mut self, count: usize) -> Option<*mut [Page<A>]> {
        let bitmap = unsafe {
            core::slice::from_raw_parts_mut::<PageMarker>(self.head as *mut _, self.page_count())
        };
        let mut found = None;
        for (i, pages) in bitmap.windows(count).enumerate() {
            if pages.iter().all(|page| page.is_free()) {
                found = Some(i);
                break;
            }
        }
        if let Some(i) = found {
            bitmap.iter_mut().enumerate().for_each(|(index, page)| {
                if index == i || (index > i && index < i + count) {
                    page.set_taken();
                    if index == i + count - 1 {
                        page.set_last();
                    }
                }
            });
            let alloc_start = self.first_page();
            // println!("----------> allocating {} at {:x}", count, (alloc_start + PAGE_SIZE * i));
            let address = (alloc_start + A * i) as *mut Page<A>;
            unsafe { Some(core::slice::from_raw_parts_mut(address, count) as *mut [Page<A>]) }
        } else {
            None
        }
    }
    /// deallocates pages based on the pointer provided
    pub fn dealloc(&mut self, page: *mut [Page<A>]) {
        assert!(!page.is_null());
        let mut page_number = self.address_to_page_index(page as *mut Page<A> as *mut usize);

        let bitmap = unsafe {
            core::slice::from_raw_parts_mut::<PageMarker>(self.head as *mut _, self.page_count())
        };
        assert!(page_number < self.page_count() - 1);

        loop {
            let page = &mut bitmap[page_number];
            if !page.is_last() && page.is_taken() {
                page.clear();
                page_number += 1;
            } else {
                assert!(page.is_last(), "Double free detected");
                page.clear();
                break;
            }
        }
    }
    /// Allocates the number of pages requested and zeros them.
    pub fn zalloc(&mut self, count: usize) -> Option<*mut [Page<A>]> {
        let pages = unsafe { self.alloc(count).unwrap().as_mut().unwrap() };
        for page in pages.iter_mut() {
            for byte in page.0.iter_mut() {
                *byte = 0;
            }
        }
        Some(pages)
    }
    /// provides the number of pages that exist
    /// assumes the bitmap takes less than a page...
    /// todo: don't do that
    pub const fn page_count(&self) -> usize {
        (self.tail - self.head) / A
    }
    /// the first page-aligned location, after S + bitmap
    pub const fn first_page(&self) -> usize {
        let bitmap_end = self.head + self.page_count() * size_of::<PageMarker>();
        align_to(bitmap_end, A)
    }
    pub fn address_to_page_index(&self, address: *mut usize) -> usize {
        assert!(!address.is_null());
        (address as usize - self.head) / A
    }
}

/// rounds the address _up_ to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}
