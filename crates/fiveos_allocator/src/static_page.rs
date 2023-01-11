use core::assert;
use core::mem::size_of;

use crate::page::bitmap::PageMarker;
// use fiveos_riscv::mmu::page_table::PAGE_SIZE;
// use fiveos_riscv::mmu::{align_address_to_page, Page};

pub struct PageContents(core::sync::atomic::AtomicU8);

/// an array of L bytes, intended to represent a page of memory.
pub struct Page<const L: usize>([u8; L]);

/// a page-based allocator backed by a memory range decided at compile time
/// S is the start point of allocatable space, E is the end
/// A is the minimum-alignment (page size)
/// Safety: this will access the underlying memory directly and dereference within the given range
/// ((E - S) / A ) - (align(S,A)-S) should be greater than 0, e.g. there should be atleast 1 allocatable
/// page after using the space starting at S as a bitmap indicating which pages are used.
pub struct StaticPageAllocator<const S: usize, const E: usize, const A: usize>;

impl<const S: usize, const E: usize, const A: usize> StaticPageAllocator<S, E, A> {
    /// Allocates the number of pages requested
    pub fn alloc(count: usize) -> Option<*mut [Page<A>]> {
        let bitmap = unsafe {
            core::slice::from_raw_parts_mut::<PageMarker>(S as *mut _, Self::page_count())
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
            let alloc_start = Self::first_page();
            // println!("----------> allocating {} at {:x}", count, (alloc_start + PAGE_SIZE * i));
            let address = (alloc_start + A * i) as *mut Page<A>;
            unsafe { Some(core::slice::from_raw_parts_mut(address, count) as *mut [Page<A>]) }
        } else {
            None
        }
    }
    /// deallocates pages based on the pointer provided
    pub fn dealloc(page: *mut [Page<A>]) {
        assert!(!page.is_null());
        let mut page_number = Self::address_to_page_index(page as *mut Page<A> as *mut usize);

        let bitmap = unsafe {
            core::slice::from_raw_parts_mut::<PageMarker>(S as *mut _, Self::page_count())
        };
        assert!(page_number < Self::page_count() - 1);

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
    pub fn zalloc(count: usize) -> Option<*mut [Page<A>]> {
        let pages = unsafe { Self::alloc(count).unwrap().as_mut().unwrap() };

        for page in pages.iter_mut() {
            for byte in page.0.iter_mut() {
                *byte = 0;
            }
        }
        Some(pages)
    }
    /// provides the number of pages that exist
    pub const fn page_count() -> usize {
        (E - S) / A
    }
    /// the first page-aligned location, after S + bitmap
    pub const fn first_page() -> usize {
        let bitmap_end = S + Self::page_count() * size_of::<PageMarker>();
        align_to(bitmap_end, A)
    }
    pub fn address_to_page_index(address: *mut usize) -> usize {
        assert!(!address.is_null());
        (address as usize - S) / A
    }
}

/// rounds the address _up_ to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}
