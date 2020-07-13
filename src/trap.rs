use crate::layout::StaticLayout;
use crate::{print, println};

#[no_mangle]
extern "C" fn rust_trap() {
    println!("entered trap handler");
    panic!();
}

pub fn setup_trap_handler() {
    let layout = StaticLayout::new();
    let ts = layout.trap_start;
    // TODO: set up trap handler
}
