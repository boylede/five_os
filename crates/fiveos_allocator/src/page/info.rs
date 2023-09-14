use core::fmt::Debug;

use fiveos_peripherals::{print, print_title, println};

/// debug info about the page allocator
pub struct PageAllocatorInfo {
    pub bitmap_start: usize,
    pub bitmap_end: usize,
    pub first_page: usize,
    pub end: usize,
    pub count: usize,
    pub size: usize,
}

impl Debug for PageAllocatorInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        print_title!(f, "Setup Memory Allocation");
        println!(f, "{} pages x {}-bytes", self.count, self.size);
        println!(
            f,
            "Allocation Table: {:x} - {:x}", self.bitmap_start, self.bitmap_end
        );
        println!(f, "Usable Pages: {:x} - {:x}", self.first_page, self.end);
        Ok(())
    }
}
