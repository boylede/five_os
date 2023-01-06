extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

pub mod bitmap;
pub mod page;

/// pointer to the first allocatable-page, i.e. the first
/// free page-aligned address in memory located after the
/// bitmap (bytemap) of all pages.
static mut ALLOC_START: usize = 0;
/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
pub const PAGE_ADDR_MAGNITIDE: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with low 12 bits set
pub const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

struct BumpPointerAlloc {
    head: usize,
    end: usize,
}

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!()
    }
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        unimplemented!()
    }
}

#[global_allocator]
static HEAP: BumpPointerAlloc = BumpPointerAlloc {
    head: 0x8800_0000,
    end: 0x9000_0000,
};

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    panic!("OOM");
}
