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

// todo: eliminate accesses to these
static mut KMEM_HEAD: *mut AllocList = null_mut();
static mut KMEM_SIZE: usize = 64;

/// debug view of initialized kernel heap info
pub struct HeapInfo {
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

pub fn init_kmem(page_allocator: &mut PageAllocator<PAGE_SIZE>) -> HeapInfo {
    unsafe {
        let size = KMEM_SIZE * PAGE_SIZE;
        let k_alloc = page_allocator.zalloc(KMEM_SIZE).unwrap();
        let start = k_alloc as *mut usize as usize;
        let end = (k_alloc as *mut usize as usize) + (size);
        KMEM_HEAD = k_alloc as *mut Page<PAGE_SIZE> as *mut AllocList;
        let kernel_heap = KMEM_HEAD.as_mut().unwrap();
        kernel_heap.set_free();
        kernel_heap.set_size(size);

        KERNEL_HEAP = BumpPointerAlloc::new(start, end);
        HeapInfo { start, end, size }
    }
}

/// prints the allocation table
pub fn kmem_print_table(uart: &mut impl Write) {
    unsafe {
        let mut head = KMEM_HEAD as *mut AllocList;
        let tail = (head).add(KMEM_SIZE) as *mut AllocList;
        while head < tail {
            {
                println!(uart, "inspecting {:x}", head as *mut u8 as usize);
                let this = head.as_ref().unwrap();
                println!(
                    uart,
                    "{:p}: Length = {:<10} Taken = {}",
                    this,
                    this.get_size(),
                    this.is_taken()
                );
            }
            let next = (head as *mut u8).add((*head).get_size());
            println!(uart, "checking next: {:x}", next as usize);
            head = next as *mut AllocList;
        }
    }
    println!(uart, "done printing alloc table");
}
