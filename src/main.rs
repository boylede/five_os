#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]
extern crate alloc;
use alloc::{boxed::Box, string::String, vec};
use core::{arch::asm, ptr::null_mut};

use five_os::{trap::TrapFrame, *};
use fiveos_allocator::{
    byte::{AllocList, BumpPointerAlloc},
    page::{bitmap::PageMarker, Page, PageAllocator},
};
use fiveos_riscv::{
    cpu::{
        self,
        registers::{misa::Misa, raw::{
            asm_get_marchid, asm_get_mepc, asm_get_mimpid, asm_get_mtvec, asm_get_mvendorid,
        }},
    },
    mmu::{
        entry::PTEntryRead,
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
    uart::{UART_BASE_ADDRESS, UART_END_ADDRESS},
};

use layout::StaticLayout;

// todo: improve how we initialize these statics
#[global_allocator]
static mut KERNEL_HEAP: BumpPointerAlloc<PAGE_SIZE> = BumpPointerAlloc::new(0, 0);

// todo: improve how we initialize these statics
static mut KERNEL_PAGE_ALLOCATOR: PageAllocator<PAGE_SIZE> = PageAllocator::uninitalized();

// todo: eliminate accesses to these
static mut KMEM_HEAD: *mut AllocList = null_mut();
static mut KMEM_SIZE: usize = 64;
static mut ALLOC_START: usize = 0;

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
    cpu::uart::Uart::<UART_BASE_ADDRESS>::new().init();
    logo::print_logo();
    print_cpu_info();

    let layout = StaticLayout::get();
    layout_sanity_check(layout);

    // Setup the kernel's page table to keep track of allocations.
    let page_allocator;
    {
        let layout = StaticLayout::get();
        let count;
        let alloc_table_start;
        let alloc_table_end;
        unsafe {
            KERNEL_PAGE_ALLOCATOR = PageAllocator::new(layout.heap_start, layout.memory_end);
            page_allocator = &mut KERNEL_PAGE_ALLOCATOR;
            count = KERNEL_PAGE_ALLOCATOR.page_count();
            alloc_table_end = layout.heap_start + count * core::mem::size_of::<PageMarker>();
            alloc_table_start = align_to(alloc_table_end, PAGE_SIZE);
            ALLOC_START = alloc_table_start;
        }
        print_title!("Setup Memory Allocation");
        println!("{} pages x {}-bytes", count, PAGE_SIZE);
        println!(
            "Allocation Table: {:x} - {:x}",
            layout.heap_start, alloc_table_end
        );
        println!(
            "Usable Pages: {:x} - {:x}",
            alloc_table_start,
            alloc_table_start + count * PAGE_SIZE
        );
    }
    // allocate some space to store a stack for the trap
    let trap_stack = page_allocator
        .zalloc(1)
        .expect("failed to initialize trap stack") as *mut u8 as usize;

    let mut kernel_memory_map: [(&str, usize, usize, EntryFlags); 16] = [
        ("", 0, 0, EntryFlags::READ),
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
        ("", 0, 0, EntryFlags::READ),
        ("", 0, 0, EntryFlags::READ),
        ("", 0, 0, EntryFlags::READ),
    ];

    // allocate pages for kernel memory, initialize bumplist/skiplist allocator
    // allocate page for kernel's page table
    unsafe {
        let k_alloc = page_allocator.zalloc(KMEM_SIZE).unwrap();
        let k_alloc_end = (k_alloc as *mut usize as usize) + (KMEM_SIZE * PAGE_SIZE);
        KMEM_HEAD = k_alloc as *mut Page<PAGE_SIZE> as *mut AllocList;
        let kmem = KMEM_HEAD.as_mut().unwrap();
        kmem.set_free();
        kmem.set_size(KMEM_SIZE * PAGE_SIZE);
        KMEM_PAGE_TABLE = page_allocator.zalloc(1).unwrap() as *mut PageTableUntyped;
        KERNEL_HEAP = BumpPointerAlloc::new(k_alloc as *mut usize as usize, k_alloc_end);

        {
            let kpt = KMEM_PAGE_TABLE as *const _ as usize;
            kernel_memory_map[0] = ("Kernel Root Page Table", kpt, kpt, EntryFlags::READ_WRITE);
        }
        {
            let start = kmem as *mut _ as usize;
            let end = start + KMEM_SIZE * PAGE_SIZE;
            kernel_memory_map[1] = ("Kernel Dynamic Memory", start, end, EntryFlags::READ_WRITE);
        }
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

    print_title!("Kernel Space Identity Map");

    for (msg, start, end, flags) in kernel_memory_map {
        if msg.len() > 0 {
            println!("{}: 0x{:x}-0x{:x} {:?}", msg, start, end, flags);
            kernel_page_table.identity_map(descriptor, start, end, flags, &mut kernel_zalloc);
        }
    }

    print_mem_bitmap();

    println!("[bookmark sigil] Leaving kinit");
    unsafe {
        asm!("csrw satp, {}", in(reg) satp_val);
        asm!("sfence.vma zero, {}", in(reg)0);
    }
}

#[no_mangle]
extern "C" fn kmain() {
    print_title!("entering kmain");

    println!("setting up UART receiver");
    PLIC.set_threshold(0);
    PLIC.enable_interrupt(10);
    PLIC.set_priority(10, 1);

    {
        printhdr!("testing allocations ");
        let k = Box::<u32>::new(100);
        println!("Boxed value = {}", &k);
        let sparkle_heart = vec![240, 159, 146, 150];
        let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();
        println!("String = {}", sparkle_heart);
        println!("\n\nAllocations of a box, vector, and string");
        kmem_print_table();
        println!("test");
    }
    println!("test 2");
    println!("\n\nEverything should now be free:");
    kmem_print_table();

    printhdr!("reached end, looping");
    loop {}
}

#[no_mangle]
extern "C" fn kinit_hart() {
    println!("entering hart kinit");
    abort();
}

////////////////////////////////////////////// todo: relocate below this line
pub fn print_cpu_info() {
    let vendor = unsafe { asm_get_mvendorid() };
    let architecture = unsafe { asm_get_marchid() };
    let implementation = unsafe { asm_get_mimpid() };
    print_title!("CPU INFO");
    println!(
        "Vendor: {:x} | Architecture: {:x} | Implementation: {:x}",
        vendor, architecture, implementation
    );
    print_misa_info();
}

pub fn print_trap_info() {
    let mepc = unsafe { asm_get_mepc() };
    println!("mepc: {:x}", mepc);
    let mtvec = unsafe { asm_get_mtvec() };
    println!("mtvec: {:x}", mtvec);
}

pub fn inspect_trap_vector() {
    printhdr!("Trap");
    let mtvec = unsafe { asm_get_mtvec() };
    if mtvec == 0 {
        println!("trap vector not initialized");
        return;
    }
    println!("trap vector: {:x}", mtvec);
    match mtvec & 0b11 {
        0b00 => println!("Direct Mode"),
        0b01 => println!("Vectored Mode"),
        0b10 => println!("Reserved Value 2 Set"),
        0b11 => println!("Reserved Value 3 Set"),
        _ => unreachable!(),
    };
}

pub fn print_page_table_untyped(table: &PageTableUntyped) {
    let descriptor = unsafe { get_global_descriptor() };
    println!("{}", table.into_dynamic_typed(&descriptor));
}

pub fn print_misa_info() {
    printhdr!("Machine Instruction Set Architecture");
    let misa = Misa::get();
    let Some(misa) = misa else {
        println!("ERROR: MISA reported unexpected value 0x{:?}", misa);
        return;
    };

    println!("Reported base width: {}", misa.xlen());

    let extensions = misa.extensions();
    print!("Extensions: ");
    for (i, letter) in Misa::EXTENSION_NAMES.chars().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            print!("{}", letter);
        }
    }
    println!();
    printhdr!("Extensions");
    for (i, desc) in Misa::EXTENSION_DESCRIPTIONS.iter().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            println!("{}", desc);
        }
    }
}

pub fn alloc_table_entry_to_page_address(entry: &mut PageMarker) -> usize {
    let alloc_start = unsafe { ALLOC_START };
    let heap_start = StaticLayout::get().heap_start;
    let page_entry = (entry as *mut _) as usize;
    alloc_start + (page_entry - heap_start) * PAGE_SIZE
}

pub fn page_table() -> (&'static mut [PageMarker]) {
    let layout = StaticLayout::get();
    let heap_start = { layout.heap_start as *mut PageMarker };
    let count = layout.heap_size / PAGE_SIZE;
    let table = unsafe { core::slice::from_raw_parts_mut(heap_start, count) };
    table
}

/// prints out the currently allocated pages
pub fn print_mem_bitmap() {
    print_title!("Allocator Bitmap");
    let page_table = page_table();
    let page_count = page_table.len();
    {
        let start = ((page_table as *const _) as *const PageMarker) as usize;
        let end = start + page_count * core::mem::size_of::<PageMarker>();
        println!("Alloc Table:\t{:x} - {:x}", start, end);
    }
    {
        let alloc_start = unsafe { ALLOC_START };
        let alloc_end = alloc_start + page_count * PAGE_SIZE;
        println!("Usable Pages:\t{:x} - {:x}", alloc_start, alloc_end);
    }
    printhdr!();
    let mut middle = false;
    let mut start = 0;
    for page in page_table.iter_mut() {
        if page.is_taken() {
            if !middle {
                let page_address = alloc_table_entry_to_page_address(page);
                print!("{:x} => ", page_address);
                middle = true;
                start = page_address;
            }
            if page.is_last() {
                let page_address = alloc_table_entry_to_page_address(page) + PAGE_SIZE - 1;
                let size = (page_address - start) / PAGE_SIZE;
                print!("{:x}: {} page(s).", page_address, size + 1);
                println!("");
                middle = false;
            }
        }
    }
    printhdr!();
    {
        let used = page_table.iter().filter(|page| page.is_taken()).count();
        println!("Allocated pages: {} = {} bytes", used, used * PAGE_SIZE);
        let free = page_count - used;
        println!("Free pages: {} = {} bytes", free, free * PAGE_SIZE);
    }
}

/// prints the allocation table
pub fn kmem_print_table() {
    unsafe {
        let mut head = KMEM_HEAD as *mut AllocList;
        let tail = (head).add(KMEM_SIZE) as *mut AllocList;
        while head < tail {
            {
                println!("inspecting {:x}", head as *mut u8 as usize);
                let this = head.as_ref().unwrap();
                println!(
                    "{:p}: Length = {:<10} Taken = {}",
                    this,
                    this.get_size(),
                    this.is_taken()
                );
            }
            let next = (head as *mut u8).add((*head).get_size());
            println!("checking next: {:x}", next as usize);
            head = next as *mut AllocList;
        }
    }
    println!("done printing alloc table");
}

/// rounds the address up to the next aligned value. if the value is already aligned, it is unchanged.
/// alignment is such that address % alignment == 0;
pub const fn align_to(address: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (address + mask) & !mask
}

pub fn layout_sanity_check(l: &StaticLayout) {
    print_title!("Static Layout Sanity Check");
    println!(
        "text:\t{:x} - {:x}\t{}-bytes",
        l.text_start,
        l.text_end,
        l.text_end - l.text_start
    );
    println!(" trap:\t{:x} - {:x}??", l.trap_start, l.text_end);
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
