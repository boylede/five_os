#![no_std]
#![no_main]
#![feature(panic_info_message, allocator_api, alloc_error_handler)]

use five_os::{mem::PAGE_SIZE, *};

use layout::StaticLayout;
use mmu::EntryFlags;

/// Our first entry point out of the assembly boot.s
#[no_mangle]
extern "C" fn kinit() {
    uart::Uart::default().init();
    logo::print_logo();
    cpu_status::print_cpu_info();
    cpu_status::print_misa_info();
    layout::layout_sanity_check();

    let layout = StaticLayout::get();
    mem::bitmap::setup();
    kmem::setup();
    mmu::setup();
    let kernel_page_table = kmem::get_page_table();

    println!("---------- Kernel Space Identity Map ----------");
    {
        // map kernel page table
        let kpt = kernel_page_table as *const mmu::PageTable as usize;
        println!("Kernel root page table: {:x}", kpt);
        mmu::identity_map_range(kernel_page_table, kpt, kpt, EntryFlags::new_rw());
    }
    {
        // map kernel's dynamic memory
        let kernel_heap = kmem::get_heap_location();
        let page_count = kmem::allocation_count();
        let end = kernel_heap + page_count * PAGE_SIZE;
        println!("Dynamic Memory: {:x} -> {:x}", kernel_heap, end);
        mmu::identity_map_range(kernel_page_table, kernel_heap, end, EntryFlags::new_rw());
    }
    {
        // map allocation 'bitmap'
        let page_count = layout.heap_size / PAGE_SIZE;
        println!(
            "Allocation bitmap: {:x} -> {:x}",
            layout.heap_start,
            layout.heap_start + page_count
        );
        mmu::identity_map_range(
            kernel_page_table,
            layout.heap_start,
            layout.heap_start + page_count,
            EntryFlags::new_re(),
        );
    }
    {
        // map kernel code
        println!(
            "Kernel code section: {:x} -> {:x}",
            layout.text_start, layout.text_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            layout.text_start,
            layout.text_end,
            EntryFlags::new_re(),
        );
    }
    {
        // map rodata
        println!(
            "Readonly data section: {:x} -> {:x}",
            layout.rodata_start, layout.rodata_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            layout.rodata_start,
            layout.rodata_end,
            // probably overlaps with text, so keep execute bit on
            EntryFlags::new_re(),
        );
    }
    {
        // map data
        println!(
            "Data section: {:x} -> {:x}",
            layout.data_start, layout.data_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            layout.data_start,
            layout.data_end,
            EntryFlags::new_rw(),
        );
    }
    {
        // map bss
        println!(
            "BSS section: {:x} -> {:x}",
            layout.bss_start, layout.bss_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            layout.bss_start,
            layout.bss_end,
            EntryFlags::new_rw(),
        );
    }
    {
        // map kernel stack
        println!(
            "Kernel stack: {:x} -> {:x}",
            layout.stack_start, layout.stack_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            layout.stack_start,
            layout.stack_end,
            EntryFlags::new_rw(),
        );
    }
    {
        // map UART
        let mm_hardware_start = 0x1000_0000;
        let mm_hardware_end = 0x1000_0100;
        println!(
            "Hardware UART: {:x} -> {:x}",
            mm_hardware_start, mm_hardware_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::new_rw(),
        );
    }
    {
        // map CLINT, MSIP
        let mm_hardware_start = 0x0200_0000;
        let mm_hardware_end = 0x0200_ffff;
        println!(
            "Hardware CLINT, MSIP: {:x} -> {:x}",
            mm_hardware_start, mm_hardware_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::new_rw(),
        );
    }
    {
        // map PLIC
        let mm_hardware_start = 0x0c00_0000;
        let mm_hardware_end = 0x0c00_2000;
        println!(
            "Hardware PLIC: {:x} -> {:x}",
            mm_hardware_start, mm_hardware_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::new_rw(),
        );
    }
    {
        // map ???
        let mm_hardware_start = 0x0c20_0000;
        let mm_hardware_end = 0x0c20_8000;
        println!(
            "Hardware ???: {:x} -> {:x}",
            mm_hardware_start, mm_hardware_end
        );
        mmu::identity_map_range(
            kernel_page_table,
            mm_hardware_start,
            mm_hardware_end,
            EntryFlags::new_rw(),
        );
    }

    println!("Finished identity map of kernel memory");
    mem::bitmap::print_mem_bitmap();
    println!("done with kinit");
}

#[no_mangle]
extern "C" fn kmain() {
    println!("entering kmain");

    println!("reached end");
    abort();
}
