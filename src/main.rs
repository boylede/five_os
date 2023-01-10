#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]

use core::arch::asm;

use five_os::{
    cpu::plic::PLIC,
    mem::{page::zalloc, PAGE_SIZE},
    mmu::page_table::untyped::PageTableUntyped,
    trap::TrapFrame,
    *,
};

use layout::StaticLayout;
use mmu::EntryFlags;

/// Our first entry point out of the assembly boot.s
#[no_mangle]
extern "C" fn kinit() {
    cpu::uart::Uart::default().init();
    logo::print_logo();
    cpu_status::print_cpu_info();
    layout::layout_sanity_check();

    let layout = StaticLayout::get();
    mem::bitmap::setup();
    kmem::setup();
    mmu::setup();
    let kernel_page_table = kmem::get_page_table();
    //let page_table_erased = kernel_page_table as *const _ as usize;

    print_title!("Kernel Space Identity Map");
    {
        // map kernel page table
        let kpt = kernel_page_table as *const PageTableUntyped as usize;
        println!("Kernel root page table: {:x}", kpt);
        kernel_page_table.identity_map(kpt, kpt, EntryFlags::READ_WRITE);
    }
    {
        // map kernel's dynamic memory
        let kernel_heap = kmem::get_heap_location();
        let page_count = kmem::allocation_count();
        let end = kernel_heap + page_count * PAGE_SIZE;
        println!("Dynamic Memory: {:x} -> {:x}  RW", kernel_heap, end);
        kernel_page_table.identity_map(kernel_heap, end, EntryFlags::READ_WRITE);
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
            layout.heap_start,
            layout.heap_start + page_count,
            EntryFlags::READ_EXECUTE,
        );
    }
    {
        // map kernel code
        println!(
            "Kernel code section: {:x} -> {:x}  RE",
            layout.text_start, layout.text_end
        );
        kernel_page_table.identity_map(
            layout.text_start,
            layout.text_end,
            EntryFlags::READ_EXECUTE,
        );
    }
    {
        // map rodata
        println!(
            "Readonly data section: {:x} -> {:x}  RE",
            layout.rodata_start, layout.rodata_end
        );
        kernel_page_table.identity_map(
            layout.rodata_start,
            layout.rodata_end,
            // probably overlaps with text, so keep execute bit on
            EntryFlags::READ_EXECUTE,
        );
    }
    {
        // map data
        println!(
            "Data section: {:x} -> {:x}  RW",
            layout.data_start, layout.data_end
        );
        kernel_page_table.identity_map(layout.data_start, layout.data_end, EntryFlags::READ_WRITE);
    }
    {
        // map bss
        println!(
            "BSS section: {:x} -> {:x}  RW",
            layout.bss_start, layout.bss_end
        );
        kernel_page_table.identity_map(layout.bss_start, layout.bss_end, EntryFlags::READ_WRITE);
    }
    {
        // map kernel stack
        println!(
            "Kernel stack: {:x} -> {:x}  RW",
            layout.stack_start, layout.stack_end
        );
        kernel_page_table.identity_map(
            layout.stack_start,
            layout.stack_end,
            EntryFlags::READ_WRITE,
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
        kernel_page_table.identity_map(mm_hardware_start, mm_hardware_end, EntryFlags::READ_WRITE);
    }
    {
        // map CLINT, MSIP
        let mm_hardware_start = 0x0200_0000;
        let mm_hardware_end = 0x0200_ffff;
        println!(
            "Hardware CLINT, MSIP: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(mm_hardware_start, mm_hardware_end, EntryFlags::READ_WRITE);
    }
    {
        // map PLIC
        let mm_hardware_start = 0x0c00_0000;
        let mm_hardware_end = 0x0c00_2000;
        println!(
            "Hardware PLIC: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(mm_hardware_start, mm_hardware_end, EntryFlags::READ_WRITE);
    }
    {
        // map ???
        let mm_hardware_start = 0x0c20_0000;
        let mm_hardware_end = 0x0c20_8000;
        println!(
            "Hardware ???: {:x} -> {:x}  RW",
            mm_hardware_start, mm_hardware_end
        );
        kernel_page_table.identity_map(mm_hardware_start, mm_hardware_end, EntryFlags::READ_WRITE);
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
        kernel_page_table.identity_map(trap_stack, trap_stack + PAGE_SIZE, EntryFlags::READ_WRITE);

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
            global_trapframe_address,
            global_trapframe_address + PAGE_SIZE,
            EntryFlags::READ_WRITE,
        );
    }

    mem::bitmap::print_mem_bitmap();

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

    printhdr!("reached end, looping");
    loop {}
}

#[no_mangle]
extern "C" fn kinit_hart() {
    println!("entering hart kinit");
    abort();
}
