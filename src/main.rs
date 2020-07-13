#![no_std]
#![no_main]

mod console;
mod cpu_status;
mod mmu;
mod page;
mod uart;
mod layout;

use mmu::Sv39Table;

#[no_mangle]
extern "C" fn kmain() {
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
