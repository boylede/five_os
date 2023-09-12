#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]
extern crate alloc;

use alloc::{boxed::Box, string::String, vec};
use core::{arch::asm, fmt::Write};

use crate::{
    global_pages::init_global_pages,
    kernel_heap::{init_kmem, inspect_heap},
    layout::LinkerLayout,
    memory_manager::init_allocator,
};
use five_os::*;
use fiveos_peripherals::{print, print_title, printhdr, println};
use fiveos_riscv::cpu::registers::{
    misa::Misa,
    raw::{asm_get_marchid, asm_get_mimpid, asm_get_mvendorid},
};
use fiveos_virtio::{plic::PLIC, Peripherals, PERIPHERALS};

mod global_pages;
mod kernel_heap;
mod memory_manager;

/// Our first entry point out of the assembly boot.s
#[no_mangle]
extern "C" fn kinit() {
    /////////////////////////////////////////////////////////////////////////////////////////
    // Init Peripherals
    /////////////////////////////////////////////////////////////////////////////////////////

    let Peripherals { mut uart } = unsafe { PERIPHERALS.take().unwrap_unchecked() };
    uart.init();

    /////////////////////////////////////////////////////////////////////////////////////////
    // Boot Prints
    /////////////////////////////////////////////////////////////////////////////////////////

    logo::print_logo(&mut uart);
    print_cpu_info(&mut uart);

    let layout = LinkerLayout::get();
    print!(uart, "{:?}", layout);

    /////////////////////////////////////////////////////////////////////////////////////////
    // Set up memory manager
    /////////////////////////////////////////////////////////////////////////////////////////

    let (mut page_allocator, memory_manager_info) = unsafe { init_allocator(&layout) };
    print!(uart, "{:?}", memory_manager_info);

    /////////////////////////////////////////////////////////////////////////////////////////
    // Set up trap stack & print
    /////////////////////////////////////////////////////////////////////////////////////////

    let trap_stack = page_allocator
        .zalloc(1)
        .expect("failed to initialize trap stack") as *mut u8 as usize;

    /////////////////////////////////////////////////////////////////////////////////////////
    // init kernel heap
    /////////////////////////////////////////////////////////////////////////////////////////

    let kernel_heap_info = unsafe { init_kmem(&mut page_allocator) };

    /////////////////////////////////////////////////////////////////////////////////////////
    // init kernel page table for entry to S-mode & enabled virtual memory
    /////////////////////////////////////////////////////////////////////////////////////////

    let kernel_memory_map =
        unsafe { init_global_pages(&layout, page_allocator, trap_stack, kernel_heap_info) };

    print!(uart, "{:?}", kernel_memory_map);

    /////////////////////////////////////////////////////////////////////////////////////////
    // Debug print page allocator
    /////////////////////////////////////////////////////////////////////////////////////////

    print!(uart, "{:?}", page_allocator);

    /////////////////////////////////////////////////////////////////////////////////////////
    // Test global alloc
    /////////////////////////////////////////////////////////////////////////////////////////

    test_allocations(&mut uart);

    /////////////////////////////////////////////////////////////////////////////////////////
    // ensure all items above are set before returning,
    // note: per internet, may not be best/proper way
    /////////////////////////////////////////////////////////////////////////////////////////

    unsafe {
        asm!("sfence.vma zero, {}", in(reg)0);
    }
}

fn test_allocations(uart: &mut impl Write) {
    println!(uart, "setting up UART receiver");
    PLIC.set_threshold(0);
    PLIC.enable_interrupt(10);
    PLIC.set_priority(10, 1);

    {
        printhdr!(uart, "testing allocations ");
        let k = Box::<u32>::new(100);
        println!(uart, "Boxed value = {}", &k);
        let sparkle_heart = vec![240, 159, 146, 150];
        let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();
        println!(uart, "String = {}", sparkle_heart);
        println!(uart, "\n\nAllocations of a box, vector, and string");

        print!(uart, "{:?}", unsafe { inspect_heap() });

        println!(uart, "test");
    }
    println!(uart, "test 2");
    println!(uart, "\n\nEverything should now be free:");
    print!(uart, "{:?}", unsafe { inspect_heap() });

    printhdr!(uart, "reached end, looping");
    loop {}
}

#[no_mangle]
extern "C" fn kinit_hart() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}

////////////////////////////////////////////// todo: relocate below this line
fn print_cpu_info(uart: &mut impl Write) {
    let vendor = unsafe { asm_get_mvendorid() };
    let architecture = unsafe { asm_get_marchid() };
    let implementation = unsafe { asm_get_mimpid() };
    print_title!(uart, "CPU INFO");
    println!(
        uart,
        "Vendor: {:x} | Architecture: {:x} | Implementation: {:x}",
        vendor,
        architecture,
        implementation
    );
    print_misa_info(uart);
}

fn print_misa_info(uart: &mut impl Write) {
    printhdr!(uart, "Machine Instruction Set Architecture");
    let misa = Misa::get();
    let Some(misa) = misa else {
        println!(uart, "ERROR: MISA reported unexpected value 0x{:?}", misa);
        return;
    };

    println!(uart, "Reported base width: {}", misa.xlen());

    let extensions = misa.extensions();
    print!(uart, "Extensions: ");
    for (i, letter) in Misa::EXTENSION_NAMES.chars().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            print!(uart, "{}", letter);
        }
    }
    println!(uart,);
    printhdr!(uart, "Extensions");
    for (i, desc) in Misa::EXTENSION_DESCRIPTIONS.iter().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            println!(uart, "{}", desc);
        }
    }
}

