use core::{fmt::Write, ptr::null_mut};

use fiveos_allocator::{
    byte::{AllocList, BumpPointerAlloc},
    page::{Page, PageAllocator},
};
use fiveos_peripherals::{print, println};
use fiveos_riscv::mmu::page_table::PAGE_SIZE;

// todo: improve how we initialize these statics
#[global_allocator]
static mut KERNEL_HEAP: BumpPointerAlloc<PAGE_SIZE> = BumpPointerAlloc::new(0, 0);

const KMEM_SIZE: usize = 64;

/// debug view of initialized kernel heap info
pub struct HeapInfo {
    pub start: usize,
    pub end: usize,
    pub size: usize,
}
/// Initialize the static global alloc for the kernel's use. 
/// 
/// ## Safety
/// Accesses static mut, expected to only run once
/// during kinit while other harts are parked
pub unsafe fn init_kmem(page_allocator: &mut PageAllocator<PAGE_SIZE>) -> HeapInfo {
        // number of bytes to allocate for initial kernel heap
        let size = KMEM_SIZE * PAGE_SIZE;
        // allocate these pages
        let k_alloc = page_allocator.zalloc(KMEM_SIZE).unwrap();
        // get resulting start, end addresses
        let start = k_alloc as *mut usize as usize;
        let end = (k_alloc as *mut usize as usize) + (size);

        let kernel_heap = (k_alloc as *mut AllocList).as_mut().unwrap();
        kernel_heap.set_free();
        kernel_heap.set_size(size);

        KERNEL_HEAP = BumpPointerAlloc::new(start, end);
        HeapInfo { start, end, size }
}

/// Provides raw access to the kernel heap allocator. 
/// Intended for use in debugging.
/// 
/// ## Safety
/// Likely not safe in any context. 
pub unsafe fn inspect_heap() -> &'static BumpPointerAlloc<PAGE_SIZE> {
    &KERNEL_HEAP
}
