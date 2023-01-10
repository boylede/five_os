use crate::layout::StaticLayout;

use crate::memory::{ALLOC_START, PAGE_SIZE, PAGE_ADDR_MAGNITIDE};
use crate::mmu::Page;
use crate::{print, println};


pub mod bitmap;
use bitmap::{page_table, PageMarker};

/// Produces a page-aligned address by adding one
/// less than the page size (4095), then masking low bits
/// to decrease the address back to the nearest page boundary
pub const fn align_address_to_page(address: usize) -> usize {
    align_power(address, PAGE_ADDR_MAGNITIDE)
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
pub const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that the number of low bits equal to power is set to zero.
pub const fn align_power(address: usize, power: usize) -> usize {
    align_to(address, 1 << power)
}

pub struct PageContents(core::sync::atomic::AtomicU8);

/// Allocates the number of pages requested
pub fn alloc(count: usize) -> Option<*mut [Page]> {
    assert!(count > 0);
    let (page_table, _) = page_table();

    let mut found = None;
    for (i, pages) in page_table.windows(count).enumerate() {
        if pages.iter().all(|page| page.is_free()) {
            found = Some(i);
            break;
        }
    }
    if let Some(i) = found {
        page_table.iter_mut().enumerate().for_each(|(index, page)| {
            if index == i || (index > i && index < i + count) {
                page.set_taken();
                if index == i + count - 1 {
                    page.set_last();
                }
            }
        });
        let alloc_start = unsafe { ALLOC_START };
        // println!("----------> allocating {} at {:x}", count, (alloc_start + PAGE_SIZE * i));
        let address = (alloc_start + PAGE_SIZE * i) as *mut Page;
        unsafe { Some(core::slice::from_raw_parts_mut(address, count) as *mut [Page]) }
    } else {
        // NonNull::dangling()
        None
        // null_mut()
        // core::ptr::null_mut()
    }
}

/// deallocates pages based on the pointer provided
pub fn dealloc(page: *mut Page) {
    assert!(!page.is_null());
    let mut page_number = address_to_page_index(page as *mut Page as *mut usize);

    let (page_table, max_index) = page_table();
    assert!(page_number < max_index);

    loop {
        let page = &mut page_table[page_number];
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
pub fn zalloc(count: usize) -> Option<*mut [Page]> {
    let pages = unsafe { alloc(count).unwrap().as_mut().unwrap() };

    for page in pages.iter_mut() {
        for byte in page.0.iter_mut() {
            *byte = 0;
        }
    }
    Some(pages)
    // Option<*mut [Page]>
    // unimplemented!()
    // let page = alloc(count) as *mut u64;
    // for i in 0..(PAGE_SIZE * count) / 8 {
    //     unsafe { *page.add(i) = 0 };
    // }
    // page as *mut usize
}

pub fn address_to_page_index(address: *mut usize) -> usize {
    assert!(!address.is_null());
    let alloc_start = unsafe { ALLOC_START };
    (address as usize - alloc_start) / PAGE_SIZE
}

pub fn page_index_to_address(index: usize) -> usize {
    let alloc_start = unsafe { ALLOC_START };
    (index * PAGE_SIZE) + alloc_start
}

pub fn alloc_table_entry_to_page_address(entry: &mut PageMarker) -> usize {
    let alloc_start = unsafe { ALLOC_START };
    let heap_start = StaticLayout::get().heap_start;
    let page_entry = (entry as *mut _) as usize;
    alloc_start + (page_entry - heap_start) * PAGE_SIZE
}
