use five_os::layout::LinkerLayout;
use fiveos_allocator::page::PageAllocator;
use fiveos_riscv::mmu::page_table::PAGE_SIZE;

// todo: improve how we initialize these statics
static mut KERNEL_PAGE_ALLOCATOR: PageAllocator<PAGE_SIZE> = PageAllocator::uninitalized();

/// Initialize page allocator
///
/// ## Safety
/// This is expected to only run once, in kinit.
pub unsafe fn init_allocator(layout: &LinkerLayout) -> &'static mut PageAllocator<PAGE_SIZE> {
    let bitmap_start = layout.heap_start;
    let end = layout.memory_end;
    let page_allocator = PageAllocator::new(bitmap_start, end);
    let page_allocator = unsafe {
        KERNEL_PAGE_ALLOCATOR = page_allocator;
        &mut KERNEL_PAGE_ALLOCATOR
    };
    page_allocator
}
