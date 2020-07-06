#![no_std]
#![no_main]

mod console;
mod cpu_status;
mod page;
mod uart;

#[no_mangle]
extern "C" fn kmain() {
    cpu_status::inspect_trap_vector();
    cpu_status::print_misa_info();
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
