#![no_std]
#![feature(
    fn_align,
    panic_info_message,
    allocator_api,
    alloc_error_handler,
    const_mut_refs
)]

use fiveos_peripherals::{println, print};
use fiveos_virtio::uart::Uart0;

// Allow testing this library
#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
extern crate test;

pub mod assembly;
pub mod layout;
pub mod logo;
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

pub trait Kernel {
    fn alloc(&mut self, count: usize) -> *mut u8;
    fn dealloc(&mut self, ptr: *mut u8);
    fn zalloc(&mut self, count: usize) -> *mut u8;
}
