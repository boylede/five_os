use core::fmt::Debug;

use fiveos_peripherals::{print, print_title, println};

extern "C" {
    static _text_start: usize;
    static _trap_start: usize;
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

/// Allows access to the global addresses PROVIDE'd in the linker map. The main reason for this structure
/// to exist is that since the locations are provided from assembly/the linker, rustc does not know the
/// addresses at compile-time so we get them at runtime instead. We could do the binding-to-address transformation
/// as .data in the assembly output, but this way allows safe rust access to the values without a bunch of
/// extern "c" accesses throughout the codebase.
pub struct LinkerLayout {
    pub text_start: usize,
    pub trap_start: usize,
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

impl LinkerLayout {
    /// creates a stack-allocated structure with all
    /// of the addresses of areas in memory.
    ///
    /// Assumes the symbols are provided by the linker,
    /// if they are not a compiler error will result.
    pub fn new() -> LinkerLayout {
        unsafe {
            LinkerLayout {
                text_start: &_text_start as *const _ as usize,
                trap_start: &_trap_start as *const _ as usize,
                text_end: &_text_end as *const _ as usize,
                global_pointer: &_global_pointer as *const _ as usize,
                rodata_start: &_rodata_start as *const _ as usize,
                rodata_end: &_rodata_end as *const _ as usize,
                data_start: &_data_start as *const _ as usize,
                data_end: &_data_end as *const _ as usize,
                bss_start: &_bss_start as *const _ as usize,
                bss_end: &_bss_end as *const _ as usize,
                memory_start: &_memory_start as *const _ as usize,
                stack_start: &_stack_start as *const _ as usize,
                stack_end: &_stack_end as *const _ as usize,
                heap_start: &_heap_start as *const _ as usize,
                heap_size: &_heap_size as *const _ as usize,
                memory_end: &_memory_end as *const _ as usize,
                trap_vector: &asm_trap_vector as *const _ as usize,
            }
        }
    }
    /// Provides the layout
    pub fn get() -> LinkerLayout {
        LinkerLayout::new()
    }
}

impl Debug for LinkerLayout {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        print_title!(f, "Static Layout Sanity Check");
        println!(
            f,
            "text:\t{:x} - {:x}\t{}-bytes",
            self.text_start,
            self.text_end,
            self.text_end - self.text_start
        );
        println!(f, " trap:\t{:x} - {:x}??", self.trap_start, self.text_end);
        println!(f, "global:\t{:x}", self.global_pointer);
        println!(
            f,
            "rodata:\t{:x} - {:x}\t{}-bytes",
            self.rodata_start,
            self.rodata_end,
            self.rodata_end - self.rodata_start
        );
        println!(
            f,
            "data:\t{:x} - {:x}\t{}-bytes",
            self.data_start,
            self.data_end,
            self.data_end - self.data_start
        );
        println!(
            f,
            "bss:\t{:x} - {:x}\t{}-bytes",
            self.bss_start,
            self.bss_end,
            self.bss_end - self.bss_start
        );
        println!(
            f,
            " stack:\t{:x} - {:x}\t{}-bytes",
            self.stack_start,
            self.stack_end,
            self.stack_end - self.stack_start
        );
        println!(
            f,
            " heap:\t{:x} - {:x}\t{}-bytes",
            self.heap_start,
            self.heap_start + self.heap_size,
            self.heap_size
        );
        Ok(())
    }
}
