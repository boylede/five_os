extern "C" {
    static _text_start: usize;
    static _trap_start: usize;
    static _trap_end: usize;
    static _text_end: usize;
    static _global_pointer: usize;
    static _rodata_start: usize;
    static _rodata_end: usize;
    static _data_start: usize;
    static _data_end: usize;
    static _bss_start: usize;
    static _bss_end: usize;
    static _memory_start: usize;
    static _stack_start: usize;
    static _stack_end: usize;
    static _heap_start: usize;
    static _heap_size: usize;
    static _memory_end: usize;
    static asm_trap_vector: usize;
}

static mut LAYOUT: Option<StaticLayout> = None;

/// Allows access to the global addresses PROVIDE'd in the linker map. The main reason for this structure
/// to exist is that since the locations are provided from assembly/the linker, rustc does not know the
/// addresses at compile-time so we get them at runtime instead. We could do the binding-to-address transformation
/// as .data in the assembly output, but this way allows safe rust access to the values without a bunch of
/// extern "c" accesses throughout the codebase.
pub struct StaticLayout {
    pub text_start: usize,
    pub trap_start: usize,
    pub trap_end: usize,
    pub text_end: usize,
    pub global_pointer: usize,
    pub rodata_start: usize,
    pub rodata_end: usize,
    pub data_start: usize,
    pub data_end: usize,
    pub bss_start: usize,
    pub bss_end: usize,
    pub memory_start: usize,
    pub stack_start: usize,
    pub stack_end: usize,
    pub heap_start: usize,
    pub heap_size: usize,
    pub memory_end: usize,
    pub trap_vector: usize,
}

impl StaticLayout {
    /// creates a stack-allocated structure with all of the addresses of areas in memory.
    /// # Unsafe
    /// Accessing a global defined outside rust is unsafe, therefor this function makes
    /// heavy use of unsafe. We also convert those addresses (pointers) to integers.
    pub fn new() -> StaticLayout {
        StaticLayout {
            text_start: unsafe { &_text_start as *const _ } as usize,
            trap_start: unsafe { &_trap_start as *const _ } as usize,
            trap_end: unsafe { &_trap_end as *const _ } as usize,
            text_end: unsafe { &_text_end as *const _ } as usize,
            global_pointer: unsafe { &_global_pointer as *const _ } as usize,
            rodata_start: unsafe { &_rodata_start as *const _ } as usize,
            rodata_end: unsafe { &_rodata_end as *const _ } as usize,
            data_start: unsafe { &_data_start as *const _ } as usize,
            data_end: unsafe { &_data_end as *const _ } as usize,
            bss_start: unsafe { &_bss_start as *const _ } as usize,
            bss_end: unsafe { &_bss_end as *const _ } as usize,
            memory_start: unsafe { &_memory_start as *const _ } as usize,
            stack_start: unsafe { &_stack_start as *const _ } as usize,
            stack_end: unsafe { &_stack_end as *const _ } as usize,
            heap_start: unsafe { &_heap_start as *const _ } as usize,
            heap_size: unsafe { &_heap_size as *const _ } as usize,
            memory_end: unsafe { &_memory_end as *const _ } as usize,
            trap_vector: unsafe { &asm_trap_vector as *const _ } as usize,
        }
    }
    /// Provides a static singleton of the layout
    pub fn get() -> &'static StaticLayout {
        unsafe {
            if LAYOUT.is_none() {
                LAYOUT = Some(StaticLayout::new());
            }
            LAYOUT.as_ref().unwrap()
        }
    }
}

use crate::{print, println};

pub fn layout_sanity_check() {
    println!("----------- Static Layout ---------------");
    let l = StaticLayout::get();
    println!(
        "text:\t{:x} - {:x}\t{}-bytes",
        l.text_start,
        l.text_end,
        l.text_end - l.text_start
    );
    println!(" trap:\t{:x} - {:x}", l.trap_start, l.trap_end);
    println!("global:\t{:x}", l.global_pointer);
    println!(
        "rodata:\t{:x} - {:x}\t{}-bytes",
        l.rodata_start,
        l.rodata_end,
        l.rodata_end - l.rodata_start
    );
    println!(
        "data:\t{:x} - {:x}\t{}-bytes",
        l.data_start,
        l.data_end,
        l.data_end - l.data_start
    );
    println!(
        "bss:\t{:x} - {:x}\t{}-bytes",
        l.bss_start,
        l.bss_end,
        l.bss_end - l.bss_start
    );
    println!(
        " stack:\t{:x} - {:x}\t{}-bytes",
        l.stack_start,
        l.stack_end,
        l.stack_end - l.stack_start
    );
    println!(
        " heap:\t{:x} - {:x}\t{}-bytes",
        l.heap_start,
        l.heap_start + l.heap_size,
        l.heap_size
    );
}
