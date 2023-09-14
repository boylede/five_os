use core::assert;
use core::fmt::Debug;
use core::mem::size_of;

use bitmap::PageMarker;
use fiveos_peripherals::{print, print_title, printhdr, println};

use self::info::PageAllocatorInfo;

pub mod bitmap;
pub mod info;

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
    pub fn info(&self) -> PageAllocatorInfo {
        let bitmap_start = self.head;
        let count = self.page_count();
        let bitmap_end = bitmap_start + count * core::mem::size_of::<PageMarker>();
        let first_page = align_to(bitmap_end, A);
        let end = self.tail;
        let size = A;
        PageAllocatorInfo {
            bitmap_start,
            bitmap_end,
            first_page,
            end,
            count,
            size,
        }
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
    pub fn marker_to_address(&self, marker: &PageMarker) -> usize {
        let alloc_start = self.first_page();
        let heap_start = self.head;
        let page_entry = (marker as *const _) as usize;
        alloc_start + (page_entry - heap_start) * A
    }
}

impl<const A: usize> Debug for PageAllocator<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        print_title!(f, "Allocator Bitmap");
        let bitmap = self.bitmap();
        let page_count = bitmap.len();
        {
            let start = ((bitmap as *const _) as *const PageMarker) as usize;
            let end = start + page_count * core::mem::size_of::<PageMarker>();
            println!(f, "Alloc Table:\t{:x} - {:x}", start, end);
        }
        {
            let alloc_start = self.first_page();
            let alloc_end = alloc_start + page_count * A;
            println!(f, "Usable Pages:\t{:x} - {:x}", alloc_start, alloc_end);
        }
        printhdr!(f,);
        let mut middle = false;
        let mut start = 0;
        for page in bitmap.iter() {
            if page.is_taken() {
                if !middle {
                    let page_address = self.marker_to_address(page);
                    print!(f, "{:x} => ", page_address);
                    middle = true;
                    start = page_address;
                }
                if page.is_last() {
                    let page_address = self.marker_to_address(page) + A - 1;
                    let size = (page_address - start) / A;
                    print!(f, "{:x}: {} page(s).", page_address, size + 1);
                    println!(f, "");
                    middle = false;
                }
            }
        }
        printhdr!(f,);
        {
            let used = bitmap.iter().filter(|page| page.is_taken()).count();
            println!(f, "Allocated pages: {} = {} bytes", used, used * A);
            let free = page_count - used;
            println!(f, "Free pages: {} = {} bytes", free, free * A);
        }
        Ok(())
    }
}

/// rounds the address _up_ to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}
