use crate::layout::StaticLayout;
use crate::{print, println};
use core::{mem::size_of, ptr::null_mut};

static mut ALLOC_START: usize = 0;
/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
pub const PAGE_ADDR_MAGNITIDE: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with low 12 bits set
pub const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

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
        self.0 |= 0b1;
    }
    pub fn set_last(&mut self) {
        self.0 |= 0b10;
    }
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    pub fn set_owner(&mut self, value: u8) {
        let mut mask = value & ((1 << 6) - 1);
        mask <<= 2;
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
            page_number += 1;
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

pub fn print_page_table() {
    println!("----------- Page Table --------------");
    let (page_table, page_count) = page_table();
    {
        let start = ((page_table as *const _) as *const Page) as usize;
        let end = start + page_count * size_of::<Page>();
        println!("Alloc Table:\t{:x} - {:x}", start, end);
    }
    {
        let alloc_start = unsafe { ALLOC_START };
        let alloc_end = alloc_start + page_count * PAGE_SIZE;
        println!("Usable Pages:\t{:x} - {:x}", alloc_start, alloc_end);
    }
    println!("   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    let mut middle = false;
    let mut start = 0;
    for page in page_table.iter_mut() {
        if page.is_taken() {
            if !middle {
                let page_address = alloc_table_entry_to_page_address(page);
                print!("{:x} => ", page_address);
                middle = true;
                start = page_address;
            }
            if page.is_last() {
                let page_address = alloc_table_entry_to_page_address(page) + PAGE_SIZE - 1;
                let size = (page_address - start) / PAGE_SIZE;
                println!("{:x}: {} page(s).", page_address, size + 1);
                middle = false;
            }
        }
    }
    println!("   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    {
        let used = page_table.iter().filter(|page| page.is_taken()).count();
        println!("Allocated pages: {} = {} bytes", used, used * PAGE_SIZE);
        let free = page_count - used;
        println!("Free pages: {} = {} bytes", free, free * PAGE_SIZE);
    }
    println!("----------------------------------------");
}

/// Setup the kernel's page table to keep track of allocations.
pub fn setup() {
    println!("----------- Dynamic Layout --------------");
    let layout = StaticLayout::get();
    let (page_table, total_page_count) = page_table();
    println!("{} pages x {}-bytes", total_page_count, PAGE_SIZE);
    for page in page_table.iter_mut() {
        page.clear();
    }

    let end_of_allocation_table = layout.heap_start + total_page_count * size_of::<Page>();
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

pub fn page_table() -> (&'static mut [Page], usize) {
    let layout = StaticLayout::get();
    let heap_start = { layout.heap_start as *mut Page };
    let count = layout.heap_size / PAGE_SIZE;
    let table = unsafe { core::slice::from_raw_parts_mut(heap_start, count) };
    (table, count)
}

pub fn address_to_page_index(address: *mut u8) -> usize {
    assert!(!address.is_null());
    let alloc_start = unsafe { ALLOC_START };
    (address as usize - alloc_start) / PAGE_SIZE
}

pub fn page_index_to_address(index: usize) -> usize {
    let alloc_start = unsafe { ALLOC_START };
    (index * PAGE_SIZE) + alloc_start
}

pub fn alloc_table_entry_to_page_address(entry: &mut Page) -> usize {
    let alloc_start = unsafe { ALLOC_START };
    let heap_start = StaticLayout::get().heap_start;
    let page_entry = (entry as *mut _) as usize;
    alloc_start + (page_entry - heap_start) * PAGE_SIZE
}
