#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]
extern crate alloc;
use core::{arch::asm, ptr::null_mut};

use alloc::{boxed::Box, string::String, vec};
use five_os::{trap::TrapFrame, *};

use fiveos_allocator::{
    byte::{AllocList, BumpPointerAlloc},
    page::{bitmap::PageMarker, Page, PageAllocator},
};
use fiveos_riscv::{
    cpu::{
        self,
        status::{
            asm_get_marchid, asm_get_mepc, asm_get_mimpid, asm_get_mtvec, asm_get_mvendorid, Misa,
        },
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
use fiveos_virtio::plic::PLIC;
use layout::StaticLayout;

// todo: improve how we initialize these statics
#[global_allocator]
static mut KERNEL_HEAP: BumpPointerAlloc<PAGE_SIZE> = BumpPointerAlloc::new(0, 0);

#[alloc_error_handler]
fn on_oom(_layout: core::alloc::Layout) -> ! {
    panic!("OOM");
}

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

/// Our first entry point out of the assembly boot.s
#[no_mangle]
extern "C" fn kinit() {
    cpu::uart::Uart::default().init();
    logo::print_logo();
    print_cpu_info();

    let layout = StaticLayout::get();
    layout_sanity_check(layout);

    // Setup the kernel's page table to keep track of allocations.
    unsafe {
        print_title!("Setup Memory Allocation");
        let layout = StaticLayout::get();
        KERNEL_PAGE_ALLOCATOR = PageAllocator::new(layout.heap_start, layout.memory_end);

        let count = KERNEL_PAGE_ALLOCATOR.page_count();
        println!("{} pages x {}-bytes", count, PAGE_SIZE);

        let end_of_allocation_table =
            layout.heap_start + count * core::mem::size_of::<PageMarker>();
        println!(
            "Allocation Table: {:x} - {:x}",
            layout.heap_start, end_of_allocation_table
        );

        let alloc_start = align_to(end_of_allocation_table, PAGE_SIZE);
        ALLOC_START = alloc_start;
        println!(
            "Usable Pages: {:x} - {:x}",
            alloc_start,
            alloc_start + count * PAGE_SIZE
        );
    }
    let page_allocator = unsafe { &mut KERNEL_PAGE_ALLOCATOR };
    // allocate pages for kernel memory, initialize bumplist/skiplist allocator
    // allocate page for kernel's page table
    unsafe {
        let k_alloc = page_allocator.zalloc(KMEM_SIZE).unwrap();
        let k_alloc_end = (k_alloc as *mut usize as usize) + (KMEM_SIZE * PAGE_SIZE);
        // assert!(!k_alloc.is_null());
        KMEM_HEAD = k_alloc as *mut Page<PAGE_SIZE> as *mut AllocList;
        let kmem = KMEM_HEAD.as_mut().unwrap();
        kmem.set_free();
        kmem.set_size(KMEM_SIZE * PAGE_SIZE);
        KMEM_PAGE_TABLE = page_allocator.zalloc(1).unwrap() as *mut PageTableUntyped;
        // KMEM_PAGE_TABLE.initialize();
        KERNEL_HEAP = BumpPointerAlloc::new(k_alloc as *mut usize as usize, k_alloc_end);
    }

    let kernel_page_table = unsafe { KMEM_PAGE_TABLE.as_mut().unwrap() };
    if !set_translation_table(TableTypes::Sv39, kernel_page_table) {
        panic!("address translation not supported on this processor.");
    }

    print_title!("Kernel Space Identity Map");
    let descriptor = unsafe { get_global_descriptor() };
    let mut kernel_zalloc =
        |count: usize| -> Option<*mut u8> { page_allocator.zalloc(count).map(|p| p as *mut u8) };
    {
        // map kernel page table
        let kpt = kernel_page_table as *const PageTableUntyped as usize;
        println!("Kernel root page table: {:x}", kpt);

        kernel_page_table.identity_map(
            descriptor,
            kpt,
            kpt,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map kernel's dynamic memory
        let kernel_heap = unsafe { KMEM_HEAD as usize };
        let page_count = unsafe { KMEM_SIZE };
        let end = kernel_heap + page_count * PAGE_SIZE;
        println!("Dynamic Memory: {:x} -> {:x}  RW", kernel_heap, end);
        kernel_page_table.identity_map(
            descriptor,
            kernel_heap,
            end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map allocation 'bitmap'
        let page_count = layout.heap_size / PAGE_SIZE;
        println!(
            "Allocation bitmap: {:x} -> {:x}  RE",
            layout.heap_start,
            layout.heap_start + page_count
        );
        kernel_page_table.identity_map(
            descriptor,
            layout.heap_start,
            layout.heap_start + page_count,
            EntryFlags::READ_EXECUTE,
            &mut kernel_zalloc,
        );
    }
    {
        // map kernel code
        println!(
            "Kernel code section: {:x} -> {:x}  RE",
            layout.text_start, layout.text_end
        );
        kernel_page_table.identity_map(
            descriptor,
            layout.text_start,
            layout.text_end,
            EntryFlags::READ_EXECUTE,
            &mut kernel_zalloc,
        );
    }
    {
        // map rodata
        println!(
            "Readonly data section: {:x} -> {:x}  RE",
            layout.rodata_start, layout.rodata_end
        );
        kernel_page_table.identity_map(
            descriptor,
            layout.rodata_start,
            layout.rodata_end,
            // probably overlaps with text, so keep execute bit on
            EntryFlags::READ_EXECUTE,
            &mut kernel_zalloc,
        );
    }
    {
        // map data
        println!(
            "Data section: {:x} -> {:x}  RW",
            layout.data_start, layout.data_end
        );
        kernel_page_table.identity_map(
            descriptor,
            layout.data_start,
            layout.data_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map bss
        println!(
            "BSS section: {:x} -> {:x}  RW",
            layout.bss_start, layout.bss_end
        );
        kernel_page_table.identity_map(
            descriptor,
            layout.bss_start,
            layout.bss_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map kernel stack
        println!(
            "Kernel stack: {:x} -> {:x}  RW",
            layout.stack_start, layout.stack_end
        );
        kernel_page_table.identity_map(
            descriptor,
            layout.stack_start,
            layout.stack_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map UART
        let mm_hardware_start = 0x1000_0000;
        let mm_hardware_end = 0x1000_0100;
        println!(
            "Hardware UART: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(
            descriptor,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map CLINT, MSIP
        let mm_hardware_start = 0x0200_0000;
        let mm_hardware_end = 0x0200_ffff;
        println!(
            "Hardware CLINT, MSIP: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(
            descriptor,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map PLIC
        let mm_hardware_start = 0x0c00_0000;
        let mm_hardware_end = 0x0c00_2000;
        println!(
            "Hardware PLIC: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(
            descriptor,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }
    {
        // map ???
        let mm_hardware_start = 0x0c20_0000;
        let mm_hardware_end = 0x0c20_8000;
        println!(
            "Hardware ???: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(
            descriptor,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );
    }

    // set up mmu satp value; todo: do this elsewhere / via zst interface
    let root_ppn = (kernel_page_table as *const _ as usize) >> 12;
    let satp_val = 8 << 60 | root_ppn;

    {
        // reserve some space for trap frames
        // and put the address in mscratch
        // (and sscratch)
        // to be used in trap.s

        // get a page for the trap stack frame
        let trap_stack = kernel_zalloc(1).expect("failed to initialize trap stack") as usize;
        println!(
            "Trap stack: {:x} -> {:x}  RW",
            trap_stack,
            trap_stack + PAGE_SIZE
        );
        kernel_page_table.identity_map(
            descriptor,
            trap_stack,
            trap_stack + PAGE_SIZE,
            EntryFlags::READ_WRITE,
            &mut kernel_zalloc,
        );

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

    print_mem_bitmap();

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
    inner_print_map(table, descriptor, 0, 0);
}

fn inner_print_map(
    table: &PageTableUntyped,
    descriptor: &PageTableDescriptor,
    base_address: usize,
    descent: usize,
) {
    let max_bits = descriptor.virtual_address_size();
    let bits_known: usize = descriptor
        .virtual_segments
        .iter()
        .take(descent + 1)
        .map(|(bits, _)| *bits)
        .sum();
    let bits_unknown = max_bits - bits_known;
    let page_size = 1 << bits_unknown;
    println!(
        "Reading pagetable located at 0x{:x}:",
        table as *const PageTableUntyped as usize
    );

    for index in 0..descriptor.size / descriptor.entry_size {
        let resulting_address = base_address + (index * page_size);
        let entry = (table.entry(index, descriptor.entry_size), descriptor);
        if entry.extract_flags().is_valid() {
            println!(
                "{}-{}: 0x{:x}-0x{:x}: {:?}",
                descent,
                index,
                resulting_address,
                resulting_address + page_size - 1,
                entry.0
            );
            if entry.extract_flags().is_branch() {
                let next = entry.address();
                let next_table = unsafe { (next as *const PageTableUntyped).as_ref().unwrap() };

                inner_print_map(next_table, descriptor, resulting_address, descent + 1);
            }
        }
    }
}

pub fn print_misa_info() {
    printhdr!("Machine Instruction Set Architecture");
    let misa = Misa::get();
    let Some(misa) = misa else {
        println!(
            "ERROR: MISA reported unexpected value 0x{:?}",
            misa
        );
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
