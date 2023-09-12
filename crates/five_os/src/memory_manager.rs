use core::fmt::Write;

use five_os::layout::StaticLayout;
use fiveos_allocator::page::{bitmap::PageMarker, PageAllocator};
use fiveos_peripherals::{print, print_title, printhdr, println};
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

pub fn print_memory_manager_info(uart: &mut impl Write, memory_manager_info: &MemoryManagerInfo) {
    {
        print_title!(uart, "Setup Memory Allocation");
        println!(
            uart,
            "{} pages x {}-bytes", memory_manager_info.count, PAGE_SIZE
        );
        println!(
            uart,
            "Allocation Table: {:x} - {:x}",
            memory_manager_info.bitmap_start,
            memory_manager_info.bitmap_end
        );
        println!(
            uart,
            "Usable Pages: {:x} - {:x}", memory_manager_info.first_page, memory_manager_info.end
        );
    }
}

pub fn alloc_table_entry_to_page_address(entry: &mut PageMarker) -> usize {
    let alloc_start = unsafe { ALLOC_START };
    let heap_start = StaticLayout::get().heap_start;
    let page_entry = (entry as *mut _) as usize;
    alloc_start + (page_entry - heap_start) * PAGE_SIZE
}

pub fn page_table() -> &'static mut [PageMarker] {
    let layout = StaticLayout::get();
    let heap_start = { layout.heap_start as *mut PageMarker };
    let count = layout.heap_size / PAGE_SIZE;
    let table = unsafe { core::slice::from_raw_parts_mut(heap_start, count) };
    table
}

/// prints out the currently allocated pages
pub fn print_mem_bitmap(uart: &mut impl Write) {
    print_title!(uart, "Allocator Bitmap");
    let page_table = page_table();
    let page_count = page_table.len();
    {
        let start = ((page_table as *const _) as *const PageMarker) as usize;
        let end = start + page_count * core::mem::size_of::<PageMarker>();
        println!(uart, "Alloc Table:\t{:x} - {:x}", start, end);
    }
    {
        let alloc_start = unsafe { ALLOC_START };
        let alloc_end = alloc_start + page_count * PAGE_SIZE;
        println!(uart, "Usable Pages:\t{:x} - {:x}", alloc_start, alloc_end);
    }
    printhdr!(uart,);
    let mut middle = false;
    let mut start = 0;
    for page in page_table.iter_mut() {
        if page.is_taken() {
            if !middle {
                let page_address = alloc_table_entry_to_page_address(page);
                print!(uart, "{:x} => ", page_address);
                middle = true;
                start = page_address;
            }
            if page.is_last() {
                let page_address = alloc_table_entry_to_page_address(page) + PAGE_SIZE - 1;
                let size = (page_address - start) / PAGE_SIZE;
                print!(uart, "{:x}: {} page(s).", page_address, size + 1);
                println!(uart, "");
                middle = false;
            }
        }
    }
    printhdr!(uart,);
    {
        let used = page_table.iter().filter(|page| page.is_taken()).count();
        println!(
            uart,
            "Allocated pages: {} = {} bytes",
            used,
            used * PAGE_SIZE
        );
        let free = page_count - used;
        println!(uart, "Free pages: {} = {} bytes", free, free * PAGE_SIZE);
    }
}
