use core::fmt::Debug;

use five_os::layout::StaticLayout;
use fiveos_allocator::page::{bitmap::PageMarker, PageAllocator};
use fiveos_peripherals::{print, print_title, println};
use fiveos_riscv::mmu::page_table::PAGE_SIZE;

// todo: improve how we initialize these statics
static mut KERNEL_PAGE_ALLOCATOR: PageAllocator<PAGE_SIZE> = PageAllocator::uninitalized();

/// location of first page of memory after
/// static kernel code, etc, and memory manager
/// data structures
static mut ALLOC_START: usize = 0;

/// debug info about the memory manager init
pub struct MemoryManagerInfo {
    bitmap_start: usize,
    bitmap_end: usize,
    first_page: usize,
    end: usize,
    count: usize,
}

impl Debug for MemoryManagerInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        print_title!(f, "Setup Memory Allocation");
        println!(f, "{} pages x {}-bytes", self.count, PAGE_SIZE);
        println!(
            f,
            "Allocation Table: {:x} - {:x}", self.bitmap_start, self.bitmap_end
        );
        println!(f, "Usable Pages: {:x} - {:x}", self.first_page, self.end);
        Ok(())
    }
}

pub unsafe fn init_allocator(
    layout: &StaticLayout,
) -> (&'static mut PageAllocator<PAGE_SIZE>, MemoryManagerInfo) {
    let bitmap_start = layout.heap_start;
    let end = layout.memory_end;
    let page_allocator = PageAllocator::new(bitmap_start, end);
    let count = page_allocator.page_count();
    let bitmap_end = bitmap_start + count * core::mem::size_of::<PageMarker>();
    let first_page = align_to(bitmap_end, PAGE_SIZE);

    let page_allocator = unsafe {
        KERNEL_PAGE_ALLOCATOR = page_allocator;
        ALLOC_START = first_page;
        &mut KERNEL_PAGE_ALLOCATOR
    };

    (
        page_allocator,
        MemoryManagerInfo {
            bitmap_start,
            bitmap_end,
            first_page,
            end,
            count,
        },
    )
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}
