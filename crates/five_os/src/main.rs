#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]
extern crate alloc;
use alloc::{boxed::Box, string::String, vec};
use fiveos_allocator::page::PageAllocator;
use core::{arch::asm, fmt::Write, ptr::null_mut};
use fiveos_peripherals::{print, print_title, printhdr, println};

use five_os::{trap::TrapFrame, *};
use fiveos_riscv::{
    cpu::registers::{
        misa::Misa,
        mtvec,
        raw::{asm_get_marchid, asm_get_mepc, asm_get_mimpid, asm_get_mvendorid},
    },
    mmu::{
        page_table::{
            descriptor::PageTableDescriptor, forty_eight::SV_FORTY_EIGHT,
            thirty_nine::SV_THIRTY_NINE, thirty_two::SV_THIRTY_TWO, untyped::PageTableUntyped,
            PAGE_SIZE,
        },
        set_translation_table, EntryFlags, TableTypes,
    },
};
use fiveos_virtio::{
    clint::{CLINT_BASE_ADDRESS, CLINT_END_ADDRESS},
    plic::{PLIC, PLIC_BASE_ADDRESS, PLIC_END_ADDRESS},
    uart::{Uart0, UART_BASE_ADDRESS, UART_END_ADDRESS},
    Peripherals, PERIPHERALS,
};

use layout::StaticLayout;

use crate::{
    kernel_heap::{init_kmem, HeapInfo, inspect_heap},
    memory_manager::init_allocator,
};

mod kernel_heap;
mod memory_manager;

/// MMU page table for kernel
static mut KMEM_PAGE_TABLE: *mut PageTableUntyped = null_mut();

/// Global that stores the type of the page table in use.
/// Provided so software can support multiple types of page tables
/// and pick between them depending on hardware support at runtime.
static mut PAGE_TABLE_TYPE: TableTypes = TableTypes::Sv39;

pub unsafe fn get_global_descriptor() -> &'static PageTableDescriptor {
    match unsafe { PAGE_TABLE_TYPE } {
        TableTypes::None => panic!("MMU not configured"),
        TableTypes::Sv32 => &SV_THIRTY_TWO,
        TableTypes::Sv39 => &SV_THIRTY_NINE,
        TableTypes::Sv48 => &SV_FORTY_EIGHT,
    }
}

#[alloc_error_handler]
fn on_oom(_layout: core::alloc::Layout) -> ! {
    panic!("OOM");
}

/// Our first entry point out of the assembly boot.s
#[no_mangle]
extern "C" fn kinit() {
    /////////////////////////////////////////////////////////////////////////////////////////
    // Init Peripherals
    /////////////////////////////////////////////////////////////////////////////////////////
    // safety: we only call this once
    let Peripherals { mut uart } = unsafe { PERIPHERALS.take().unwrap_unchecked() };
    uart.init();

    /////////////////////////////////////////////////////////////////////////////////////////
    // Boot Prints
    /////////////////////////////////////////////////////////////////////////////////////////

    logo::print_logo(&mut uart);
    print_cpu_info(&mut uart);

    let layout = StaticLayout::get();
    layout_sanity_check(&mut uart, layout);

    /////////////////////////////////////////////////////////////////////////////////////////
    // Set up memory manager
    /////////////////////////////////////////////////////////////////////////////////////////
    // Setup the kernel's page table to keep track of allocations.
    let (mut page_allocator, memory_manager_info) = unsafe { init_allocator(layout) };
    print!(uart, "{:?}", memory_manager_info);
    /////////////////////////////////////////////////////////////////////////////////////////
    // Set up trap stack & print
    /////////////////////////////////////////////////////////////////////////////////////////
    // allocate some space to store a stack for the trap
    let trap_stack = page_allocator
        .zalloc(1)
        .expect("failed to initialize trap stack") as *mut u8 as usize;

    /////////////////////////////////////////////////////////////////////////////////////////
    // Hard-coded info about kernel's memory use
    /////////////////////////////////////////////////////////////////////////////////////////
    let mut kernel_memory_map: [(&str, usize, usize, EntryFlags); 16] = [
        // dynamic entry for kernel page table
        ("", 0, 0, EntryFlags::READ),
        // dynamic entry for further kernel pages
        ("", 0, 0, EntryFlags::READ),
        (
            "Allocation Bitmap",
            layout.heap_start,
            layout.heap_start + (layout.heap_size / PAGE_SIZE),
            EntryFlags::READ_EXECUTE,
        ),
        (
            "Kernel Code Section",
            layout.text_start,
            layout.text_end,
            EntryFlags::READ_EXECUTE,
        ),
        (
            "Readonly Data Section",
            layout.rodata_start,
            layout.rodata_end,
            EntryFlags::READ_EXECUTE,
        ),
        (
            "Data Section",
            layout.data_start,
            layout.data_end,
            EntryFlags::READ_WRITE,
        ),
        (
            "BSS section",
            layout.bss_start,
            layout.bss_end,
            EntryFlags::READ_WRITE,
        ),
        (
            "Kernel Stack",
            layout.stack_start,
            layout.stack_end,
            EntryFlags::READ_WRITE,
        ),
        (
            "Hardware UART",
            UART_BASE_ADDRESS,
            UART_END_ADDRESS,
            EntryFlags::READ_WRITE,
        ),
        (
            "Hardware CLINT, MSIP",
            CLINT_BASE_ADDRESS,
            CLINT_END_ADDRESS,
            EntryFlags::READ_WRITE,
        ),
        (
            "Hardware PLIC",
            PLIC_BASE_ADDRESS,
            PLIC_END_ADDRESS,
            EntryFlags::READ_WRITE,
        ),
        (
            "Hardware ????",
            0x0c20_0000,
            0x0c20_8000,
            EntryFlags::READ_WRITE,
        ),
        (
            "Trap stack",
            trap_stack,
            trap_stack + PAGE_SIZE,
            EntryFlags::READ_WRITE,
        ),
        // unused entries
        ("", 0, 0, EntryFlags::READ),
        ("", 0, 0, EntryFlags::READ),
        ("", 0, 0, EntryFlags::READ),
    ];

    /////////////////////////////////////////////////////////////////////////////////////////
    // init kernel heap
    /////////////////////////////////////////////////////////////////////////////////////////

    // allocate pages for kernel memory, initialize bumplist/skiplist allocator
    // allocate page for kernel's page table
    let kernel_heap_info = unsafe {init_kmem(&mut page_allocator)};

    /////////////////////////////////////////////////////////////////////////////////////////
    // init kernel page table for entry to S-mode & enabled virtual memory
    /////////////////////////////////////////////////////////////////////////////////////////
    let kernel_page_table = unsafe {
        let kpt = page_allocator.zalloc(1).unwrap() as *mut PageTableUntyped;
        let kpt_int = kpt as *const usize as usize;
        KMEM_PAGE_TABLE = kpt;
        kpt_int
    };

    /////////////////////////////////////////////////////////////////////////////////////////
    // add kernel dynamic memory info to memory map
    /////////////////////////////////////////////////////////////////////////////////////////
    {
        let kpt = kernel_page_table;
        kernel_memory_map[0] = ("Kernel Root Page Table", kpt, kpt, EntryFlags::READ_WRITE);
    }
    {
        let HeapInfo { start, end, .. } = kernel_heap_info;
        kernel_memory_map[1] = ("Kernel Dynamic Memory", start, end, EntryFlags::READ_WRITE);
    }

    let kernel_page_table = unsafe { KMEM_PAGE_TABLE.as_mut().unwrap() };
    if !set_translation_table(TableTypes::Sv39, kernel_page_table) {
        panic!("address translation not supported on this processor.");
    }

    // set up mmu satp value; todo: do this elsewhere / via zst interface
    let root_ppn = (kernel_page_table as *const _ as usize) >> 12;
    let satp_val = 8 << 60 | root_ppn;
    let descriptor = unsafe { get_global_descriptor() };
    let mut kernel_zalloc =
        |count: usize| -> Option<*mut u8> { page_allocator.zalloc(count).map(|p| p as *mut u8) };
    {
        // reserve some space for trap frames
        // and put the address in mscratch
        // (and sscratch)
        // to be used in trap.s

        // get a page for the trap stack frame

        let global_trapframe_address = unsafe {
            let frame: &mut TrapFrame = &mut trap::GLOBAL_TRAPFRAMES[0];
            frame.satp = satp_val;
            frame.trap_stack = (trap_stack + PAGE_SIZE) as *mut _;
            let global_trapframe_address = frame as *mut TrapFrame as usize;

            asm!("csrw mscratch, {}", in(reg) global_trapframe_address);
            asm!("csrw sscratch, {}", in(reg) global_trapframe_address);
            global_trapframe_address
        };

        kernel_page_table.identity_map(
            descriptor,
            global_trapframe_address,
            global_trapframe_address + PAGE_SIZE,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }

    print_title!(uart, "Kernel Space Identity Map");

    for (msg, start, end, flags) in kernel_memory_map {
        if msg.len() > 0 {
            println!(uart, "{}: 0x{:x}-0x{:x} {:?}", msg, start, end, flags);
            kernel_page_table.identity_map(descriptor, start, end, flags, &mut kernel_zalloc);
        }
    }

    print!(uart, "{:?}", page_allocator);

    test_allocations(&mut uart);
    unsafe {
        asm!("csrw satp, {}", in(reg) satp_val);
        asm!("sfence.vma zero, {}", in(reg)0);
    }
}

#[no_mangle]
extern "C" fn kmain() {
    // Safety, this function doesn't get called, will be deleted
    let mut uart = unsafe { Uart0::new() };
    println!(uart, "entered KMAIN");
    loop {
        unsafe {
            asm!("wfi");
        }
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
        
        print!(uart, "{:?}", unsafe {inspect_heap()});
        
        println!(uart, "test");
    }
    println!(uart, "test 2");
    println!(uart, "\n\nEverything should now be free:");
    print!(uart, "{:?}", unsafe {inspect_heap()});

    printhdr!(uart, "reached end, looping");
    loop {}
}

#[no_mangle]
extern "C" fn kinit_hart() -> ! {
    // println!("entering hart kinit");
    loop {
        unsafe { asm!("wfi") };
    }
}

////////////////////////////////////////////// todo: relocate below this line
pub fn print_cpu_info(uart: &mut impl Write) {
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

pub fn print_trap_info(uart: &mut impl Write) {
    let mepc = unsafe { asm_get_mepc() };
    println!(uart, "mepc: {:x}", mepc);
    let mtvec = unsafe { mtvec::read() };
    println!(uart, "mtvec: {:x}", mtvec);
}

pub fn inspect_trap_vector(uart: &mut impl Write) {
    printhdr!(uart, "Trap");
    let mtvec = unsafe { mtvec::read() };
    if mtvec == 0 {
        println!(uart, "trap vector not initialized");
        return;
    }
    println!(uart, "trap vector: {:x}", mtvec);
    match mtvec & 0b11 {
        0b00 => println!(uart, "Direct Mode"),
        0b01 => println!(uart, "Vectored Mode"),
        0b10 => println!(uart, "Reserved Value 2 Set"),
        0b11 => println!(uart, "Reserved Value 3 Set"),
        _ => unreachable!(),
    };
}

pub fn print_page_table_untyped(uart: &mut impl Write, table: &PageTableUntyped) {
    let descriptor = unsafe { get_global_descriptor() };
    println!(uart, "{}", table.into_dynamic_typed(&descriptor));
}

pub fn print_misa_info(uart: &mut impl Write) {
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

pub fn layout_sanity_check(uart: &mut impl Write, l: &StaticLayout) {
    print_title!(uart, "Static Layout Sanity Check");
    println!(
        uart,
        "text:\t{:x} - {:x}\t{}-bytes",
        l.text_start,
        l.text_end,
        l.text_end - l.text_start
    );
    println!(uart, " trap:\t{:x} - {:x}??", l.trap_start, l.text_end);
    println!(uart, "global:\t{:x}", l.global_pointer);
    println!(
        uart,
        "rodata:\t{:x} - {:x}\t{}-bytes",
        l.rodata_start,
        l.rodata_end,
        l.rodata_end - l.rodata_start
    );
    println!(
        uart,
        "data:\t{:x} - {:x}\t{}-bytes",
        l.data_start,
        l.data_end,
        l.data_end - l.data_start
    );
    println!(
        uart,
        "bss:\t{:x} - {:x}\t{}-bytes",
        l.bss_start,
        l.bss_end,
        l.bss_end - l.bss_start
    );
    println!(
        uart,
        " stack:\t{:x} - {:x}\t{}-bytes",
        l.stack_start,
        l.stack_end,
        l.stack_end - l.stack_start
    );
    println!(
        uart,
        " heap:\t{:x} - {:x}\t{}-bytes",
        l.heap_start,
        l.heap_start + l.heap_size,
        l.heap_size
    );
}
