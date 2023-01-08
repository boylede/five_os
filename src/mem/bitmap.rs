use core::mem::size_of;

use crate::layout::StaticLayout;
use crate::mem::page::{align_address_to_page, alloc_table_entry_to_page_address};
use crate::mem::{ALLOC_START, PAGE_SIZE};
use crate::{print, println};

#[repr(transparent)]
pub struct PageMarker {
    flags: Pageflags,
}

impl PageMarker {
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

#[repr(transparent)]
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

/// Setup the kernel's page table to keep track of allocations.
pub fn setup() {
    println!("----------- Dynamic Layout --------------");
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

pub fn page_table() -> (&'static mut [PageMarker], usize) {
    let layout = StaticLayout::get();
    let heap_start = { layout.heap_start as *mut PageMarker };
    let count = layout.heap_size / PAGE_SIZE;
    let table = unsafe { core::slice::from_raw_parts_mut(heap_start, count) };
    (table, count)
}

/// prints out the currently allocated pages
pub fn print_mem_bitmap() {
    println!("----------- Allocator Bitmap --------------");
    let (page_table, page_count) = page_table();
    {
        let start = ((page_table as *const _) as *const PageMarker) as usize;
        let end = start + page_count * size_of::<PageMarker>();
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
                print!("{:x}: {} page(s).", page_address, size + 1);
                println!("");
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
