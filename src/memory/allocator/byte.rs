extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

use crate::kmem::{kfree, kzmalloc};

use crate::{print, println};

struct BumpPointerAlloc {
    head: usize,
    end: usize,
}

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        kzmalloc(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        println!("dropping {:x}", ptr as usize);
        kfree(ptr);
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
