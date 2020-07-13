use crate::layout::StaticLayout;
use crate::{print, println};
use core::{mem::size_of, ptr::null_mut};

static mut ALLOC_START: usize = 0;
/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
const PAGE_ADDR_MAGNITIDE: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with all used bits set
const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

/// Produces a page-aligned address by adding one
/// less than the page size (4095), then masking low bits
/// to decrease the address back to the nearest page boundary
pub const fn align_address(address: usize) -> usize {
    (address + PAGE_ADDR_MASK) & !PAGE_ADDR_MASK
}

pub const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}

pub const fn align_power(address: usize, power: usize) -> usize {
    let mask = (1 << power) - 1;
    (address + mask) & !mask
}



pub struct Page {
    flags: Pageflags,
}

impl Page {
    pub fn is_free(&self) -> bool {
        self.flags.is_empty()
    }
    pub fn is_taken(&self) -> bool {
        !self.flags.is_empty()
    }
    pub fn is_last(&self) -> bool {
        self.flags.is_last()
    }
    pub fn clear(&mut self) {
        self.flags.clear();
    }
    pub fn set_taken(&mut self) {
        self.flags.set_taken();
    }
    pub fn set_last(&mut self) {
        self.flags.set_last();
    }
}

struct Pageflags(u8);

impl Pageflags {
    pub fn is_taken(&self) -> bool {
        self.0 & 0b1 == 0b1
    }
    pub fn is_last(&self) -> bool {
        self.0 & 0b10 == 0b10
    }
    pub fn is_empty(&self) -> bool {
        self.0 & 0b1 == 0b0
    }
    pub fn owner(&self) -> u8 {
        self.0 >> 2
    }
    pub fn set_taken(&mut self) {
        self.0 = self.0 | 0b1;
    }
    pub fn set_last(&mut self) {
        self.0 = self.0 | 0b10;
    }
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    pub fn set_owner(&mut self, value: u8) {
        let mut mask = value & (1 << 6) - 1;
        mask = mask << 2;
        self.0 = (self.0 & 0b11) | mask;
    }
}

/// Allocates the number of pages requested
pub fn alloc(count: usize) -> *mut u8 {
    assert!(count > 0);
    let (page_table, _) = page_table();

    let mut found = None;
    for (i, pages) in page_table.windows(count).enumerate() {
        if pages.iter().all(|page| page.is_free()) {
            found = Some(i);
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
        (alloc_start + PAGE_SIZE * i) as *mut u8
    } else {
        null_mut()
    }
}

/// deallocates pages based on the pointer provided
pub fn dealloc(page: *mut u8) {
    assert!(!page.is_null());
    let mut page_number = address_to_page_index(page);

    let (page_table, max_index) = page_table();
    assert!(page_number < max_index);

    loop {
        let page = &mut page_table[page_number];
        if !page.is_last() && page.is_taken() {
            page.clear();
            page_number = page_number + 1;
        } else {
            assert!(page.is_last(), "Double free detected");
            page.clear();
            break;
        }
    }
}

/// Allocates the number of pages requested and zeros them.
pub fn zalloc(count: usize) -> *mut u8 {
    let page = alloc(count) as *mut u64;
    for i in 0..(PAGE_SIZE * count) / 8 {
        unsafe { *page.add(i) = 0 };
    }
    page as *mut u8
}

pub fn print_page_table(table: &[Page]) {
    let layout = StaticLayout::new();
    let heap_size = layout.heap_size;
    let heap_start = layout.heap_start;
    let total_page_count = heap_size / PAGE_SIZE;
    assert!(total_page_count > 0);
    let mut begining = heap_start as *const Page;
    let end = unsafe { begining.add(total_page_count) };
    let allocation_beginning = unsafe { ALLOC_START };
    let allocation_end = allocation_beginning + total_page_count * PAGE_SIZE;

    println!();
    println!("Page Allocation Table");
    println!("Meta: {:p} - {:p}", begining, end);
    println!(
        "Phys: {:#04x} - {:#04x}",
        allocation_beginning, allocation_end
    );
    println!("----------------------------------------");
    let mut index = 0;
    while begining < end {
        if (unsafe { begining.as_ref().unwrap() }).is_taken() {
            //
        }
    }
}

/// Setup the kernel's page table to keep track of allocations.
pub fn setup() {
    let layout = StaticLayout::new();
    let total_page_count = layout.heap_size / PAGE_SIZE;
    println!("setting up byte-map for {} pages", total_page_count);
    let page_table = layout.heap_start as *mut Page;
    for i in 0..total_page_count {
        let current_page = unsafe { page_table.add(i).as_mut() }.unwrap();
        current_page.clear();
    }
    unsafe {
        ALLOC_START = align_address(layout.heap_start + total_page_count * size_of::<Page>())
    };
}

pub fn page_table() -> (&'static mut [Page], usize) {
    let layout = StaticLayout::new();
    let heap_start = { layout.heap_start as *mut Page };
    let count = layout.heap_size / PAGE_SIZE;
    let table = unsafe { core::slice::from_raw_parts_mut(heap_start, count) };
    (table, count)
}

pub fn address_to_page_index(address: *mut u8) -> usize {
    assert!(!address.is_null());
    let alloc_start = unsafe {ALLOC_START};
    (address as usize - alloc_start) / PAGE_SIZE
}