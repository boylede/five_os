#![no_std]
#![feature(fn_align, panic_info_message, allocator_api, alloc_error_handler, const_mut_refs)]

pub mod console;
pub mod cpu;
pub mod cpu_status;
pub mod kmem;
pub mod layout;
pub mod logo;
pub mod mem;
pub mod mmu;
pub mod process;
pub mod trap;

#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("aborting: {}", info);
    abort();
}

#[inline(never)]
#[no_mangle]
pub extern "C" fn abort() -> ! {
    use core::sync::atomic::{self, Ordering};
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
