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

use mmu::Sv39Table;

#[no_mangle]
extern "C" fn kmain() {
    logo::print_logo();
    mmu::test();
    page::setup();
    cpu_status::print_cpu_info();
    cpu_status::print_misa_info();

    layout::layout_sanity_check();
    cpu_status::setup_trap();
    cpu_status::print_trap_info();
    cpu_status::inspect_trap_vector();
    
    let table = unsafe { mmu::setup().as_mut().unwrap() };
    table.alloc(4);
    table.alloc(1);
    table.alloc(8);
    table.alloc(1);
    mmu::print_page_table(table);
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
