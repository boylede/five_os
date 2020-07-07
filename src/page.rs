use crate::{print, println};

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

static mut ALLOC_START: usize = 0;
/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
const PAGE_ADDR_MAGNITIDE: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with all used bits set
const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

/// Produces a page-aligned address by adding one
/// less than the page size (4095), then masking low bits
/// to decrease the address back to the nearest page boundary
pub const fn align_address(address: usize) -> usize {
    (address + PAGE_ADDR_MASK) & !PAGE_ADDR_MASK
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

pub fn alloc(count: usize) -> *mut u8 {
    assert!(count > 0);
    
    let heap_size = unsafe {HEAP_SIZE};
    let heap_start = unsafe {HEAP_START};
    let total_page_count = heap_size / PAGE_SIZE;
    let first_page = heap_size as *mut Page;
    for i in 0..total_page_count - count {
        let mut found = false;
        let current_page = unsafe {first_page.add(i).as_ref().unwrap()};
        if current_page.is_free() {
            found = true;
            for j in i..i + count {
                let next_page = unsafe {first_page.add(j).as_ref().unwrap()};
                if next_page.is_taken() {
                    found = true;
                    break;
                }
            }
            if found {
                return (unsafe {ALLOC_START} + PAGE_SIZE * i) as *mut u8;
            }
        }
    }
    panic!("out of memory");
}

pub fn dealloc(page: *mut u8) {
    assert!(!page.is_null());
    let heap_start = unsafe {HEAP_START};
    let page_number = (page as usize - unsafe { ALLOC_START}) / PAGE_SIZE;
    let mut entry = (heap_start + page_number) as *mut Page;
    // let mut entry = unsafe {entry.as_mut().unwrap()};
    while !(*entry).is_last() && (*entry).is_taken() {
        (*entry).clear();
    }
}

pub fn zalloc(count: usize) -> *mut u8 {
    let page = alloc(count);
    for i in 0..PAGE_SIZE {
        unsafe { *page.add(i) = 0 };
    }
    page
}

pub fn print_page_table(table: &[Page]) {
    let total_page_count = unsafe { HEAP_SIZE } / PAGE_SIZE;
    let mut begining = unsafe { HEAP_START } as *const Page;
    let end = unsafe { begining.add(total_page_count) };
    let allocation_beginning = unsafe { ALLOC_START };
    let allocation_end = allocation_beginning + total_page_count * PAGE_SIZE;

    println!();
    println!("Page Allocation Table");
    println!("Meta: {:p} - {:p}", begining, end);
    println!("Phys: {:#04x} - {:#04x}", allocation_beginning, allocation_end);
    println!("----------------------------------------");

}

pub fn setup() {
    unimplemented!()
}
