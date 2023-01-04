extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

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
