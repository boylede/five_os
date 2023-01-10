use core::mem::size_of;

use crate::layout::StaticLayout;

use crate::{print, print_title, println};

pub mod bitmap;
use bitmap::{page_table, PageMarker};
use fiveos_riscv::mmu::page_table::PAGE_SIZE;
use fiveos_riscv::mmu::{align_address_to_page, Page};

/// pointer to the first allocatable-page, i.e. the first
/// free page-aligned address in memory located after the
/// bitmap (bytemap) of all pages.
static mut ALLOC_START: usize = 0;

/// Setup the kernel's page table to keep track of allocations.
pub fn setup() {
    print_title!("Setup Memory Allocation");
    let layout = StaticLayout::get();
    let (page_table, total_page_count) = page_table();
    println!("{} pages x {}-bytes", total_page_count, PAGE_SIZE);
    for page in page_table.iter_mut() {
        page.clear();
    }

    let end_of_allocation_table = layout.heap_start + total_page_count * size_of::<PageMarker>();
    println!(
        "Allocation Table: {:x} - {:x}",
        layout.heap_start, end_of_allocation_table
    );

    let alloc_start = unsafe {
        ALLOC_START = align_address_to_page(end_of_allocation_table);
        ALLOC_START
    };
    println!(
        "Usable Pages: {:x} - {:x}",
        alloc_start,
        alloc_start + total_page_count * PAGE_SIZE
    );
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
