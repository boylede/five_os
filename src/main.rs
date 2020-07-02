#![no_std]
#![no_main]

mod console;
mod uart;

#[no_mangle]
extern "C" fn kmain() {
    println!("Entered Rust Kernel");
    let mut console = console::Console::new();
    console.run();
    panic!("Finished");
}

#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("aborting: {}", info);
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
