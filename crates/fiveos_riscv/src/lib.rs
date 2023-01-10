#![no_std]
#![feature(
    fn_align,
    panic_info_message,
    allocator_api,
    alloc_error_handler,
    const_mut_refs
)]

pub mod cpu;
pub mod mmu;
