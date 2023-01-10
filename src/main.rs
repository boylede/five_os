#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]
extern crate alloc;
use core::arch::asm;

use alloc::{boxed::Box, string::String, vec};
use five_os::{
    memory::{allocator::page::zalloc, get_global_descriptor},
    trap::TrapFrame,
    *,
};

use fiveos_riscv::{
    cpu::{
        self,
        status::{
            asm_get_marchid, asm_get_mepc, asm_get_mimpid, asm_get_misa, asm_get_mtvec,
            asm_get_mvendorid, get_base_width, set_trap_vector, EXTENSION_DESCRIPTIONS,
            EXTENSION_NAMES,
        },
    },
    mmu::{
        entry::PTEntryRead,
        page_table::{descriptor::PageTableDescriptor, untyped::PageTableUntyped, PAGE_SIZE},
        EntryFlags,
    },
};
use fiveos_virtio::plic::PLIC;
use layout::StaticLayout;

/// Our first entry point out of the assembly boot.s
#[no_mangle]
extern "C" fn kinit() {
    cpu::uart::Uart::default().init();
    logo::print_logo();
    print_cpu_info();
    layout::layout_sanity_check();

    let layout = StaticLayout::get();
    memory::allocator::page::setup();
    kmem::setup();
    memory::setup();
    let kernel_page_table = kmem::get_page_table();
    //let page_table_erased = kernel_page_table as *const _ as usize;

    print_title!("Kernel Space Identity Map");
    let descriptor = unsafe { get_global_descriptor() };
    {
        // map kernel page table
        let kpt = kernel_page_table as *const PageTableUntyped as usize;
        println!("Kernel root page table: {:x}", kpt);

        kernel_page_table.identity_map(descriptor, kpt, kpt, EntryFlags::READ_WRITE, zalloc);
    }
    {
        // map kernel's dynamic memory
        let kernel_heap = kmem::get_heap_location();
        let page_count = kmem::allocation_count();
        let end = kernel_heap + page_count * PAGE_SIZE;
        println!("Dynamic Memory: {:x} -> {:x}  RW", kernel_heap, end);
        kernel_page_table.identity_map(
            descriptor,
            kernel_heap,
            end,
            EntryFlags::READ_WRITE,
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
            zalloc,
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
        // todo: this could be in kmem init instead
        let trap_stack =
            zalloc(1).expect("failed to initialize trap stack") as *mut [_] as *mut u8 as usize;
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
            zalloc,
        );

        let global_trapframe_address = unsafe {
            // Safety: we are accessing a static mut. we are safe in kinit
            // because we are the only thing running
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
            zalloc,
        );
    }

    memory::allocator::page::bitmap::print_mem_bitmap();

    // print_map(kernel_page_table);

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
        println!("Boxed address = {:x}", Box::leak(k) as *const _ as usize);
        let sparkle_heart = vec![240, 159, 146, 150];
        let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();
        println!("String = {}", sparkle_heart);
        println!("\n\nAllocations of a box, vector, and string");
        kmem::print_table();
        println!("test");
    }
    println!("test 2");
    println!("\n\nEverything should now be free:");
    kmem::print_table();

    printhdr!("reached end, looping");
    loop {}
}

#[no_mangle]
extern "C" fn kinit_hart() {
    println!("entering hart kinit");
    abort();
}

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
////////////////////////////////////////////// todo: relocate
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
    // let page_size = 1 << (12+bits_known);
    // println!("memory region described by each entry is: 0x{:x}-bytes", page_size);

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
                // println!("branching");
                let next = entry.address();
                let next_table = unsafe { (next as *const PageTableUntyped).as_ref().unwrap() };

                inner_print_map(next_table, descriptor, resulting_address, descent + 1);

                // println!("rejoining");
            } else {
                // println!("{}-{}: 0x{:x}-0x{:x}: {:?}", descent, index, resulting_address,resulting_address+page_size-1, entry);
            }
        } else {
            // println!("{}-{}: 0x{:x}-0x{:x}: not mapped.", descent, index, resulting_address, resulting_address+page_size-1);
        }
    }
}

pub fn print_misa_info() {
    printhdr!("Machine Instruction Set Architecture");
    let misa = unsafe { asm_get_misa() };
    let xlen = {
        let mut misa: i64 = misa as i64;
        // if sign bit is 0, XLEN is 32
        if misa > 0 {
            32
        } else {
            // shift misa over 1 bit to check next-highest bit
            misa <<= 1;
            // if new sign bit is 0, XLEN is 64
            if misa > 0 {
                64
            } else {
                // both high bits are 1, so xlen is 128
                128
            }
        }
    };
    let checked_width = get_base_width();
    if xlen != checked_width {
        println!(
            "ERROR: MISA reports different base width than empirically found: {} vs {}",
            xlen, checked_width
        );
    } else {
        println!("Base ISA Width: {}", xlen);
    }

    let extensions = misa & 0x01FF_FFFF;
    print!("Extensions: ");
    for (i, letter) in EXTENSION_NAMES.chars().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            print!("{}", letter);
        }
    }
    println!();
    printhdr!("Extensions");
    for (i, desc) in EXTENSION_DESCRIPTIONS.iter().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            println!("{}", desc);
        }
    }
}

pub fn setup_trap() {
    let address = StaticLayout::get().trap_vector;
    let mask = address & 0b11;
    if mask != 0 {
        panic!("Trap vector not aligned to 4-byte boundary: {:x}", address);
    }
    set_trap_vector(address);
}
