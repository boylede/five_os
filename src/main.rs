#![no_std]
#![no_main]

mod console;
mod cpu_status;
mod kmem;
mod layout;
mod logo;
mod mmu;
mod page;
mod trap;
mod uart;

#[no_mangle]
extern "C" fn kmain() {
    logo::print_logo();
    cpu_status::print_cpu_info();
    cpu_status::print_misa_info();
    layout::layout_sanity_check();
    mmu::test();
    page::setup();
    kmem::setup();
    page::alloc(64);
    page::alloc(1);

    page::print_page_table();
    println!("reached end");
    abort();
}

#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("aborting: {}", info);
    abort();
}

#[inline(never)]
#[no_mangle]
extern "C" fn abort() -> ! {
    use core::sync::atomic::{self, Ordering};
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
