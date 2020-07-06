#![no_std]
#![no_main]

mod console;
mod cpu_status;
mod page;
mod uart;

use page::Sv39Table;

#[no_mangle]
extern "C" fn kmain() {
    cpu_status::setup_trap();
    cpu_status::inspect_trap_vector();
    cpu_status::print_misa_info();
    let table = unsafe { page::setup().as_ref().unwrap() };
    page::print_page_table(table);
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
