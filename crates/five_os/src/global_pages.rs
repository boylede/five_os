use core::{arch::asm, fmt::Debug, ptr::null_mut};
use fiveos_allocator::page::PageAllocator;

use crate::{kernel_heap::HeapInfo, layout::LinkerLayout};
use five_os::{trap::TrapFrame, *};
use fiveos_peripherals::{print, print_title, println};
use fiveos_riscv::mmu::{
    page_table::{
        descriptor::PageTableDescriptor, forty_eight::SV_FORTY_EIGHT, thirty_nine::SV_THIRTY_NINE,
        thirty_two::SV_THIRTY_TWO, untyped::PageTableUntyped, PAGE_SIZE,
    },
    set_translation_table, EntryFlags, TableTypes,
};
use fiveos_virtio::{
    clint::{CLINT_BASE_ADDRESS, CLINT_END_ADDRESS},
    plic::{PLIC_BASE_ADDRESS, PLIC_END_ADDRESS},
    uart::{UART_BASE_ADDRESS, UART_END_ADDRESS},
};

/// MMU page table for kernel
static mut KMEM_PAGE_TABLE: *mut PageTableUntyped = null_mut();

/// Global that stores the type of the page table in use.
/// Provided so software can support multiple types of page tables
/// and pick between them depending on hardware support at runtime.
static mut PAGE_TABLE_TYPE: TableTypes = TableTypes::Sv39;

pub unsafe fn get_global_descriptor() -> &'static PageTableDescriptor {
    match PAGE_TABLE_TYPE {
        TableTypes::None => panic!("MMU not configured"),
        TableTypes::Sv32 => &SV_THIRTY_TWO,
        TableTypes::Sv39 => &SV_THIRTY_NINE,
        TableTypes::Sv48 => &SV_FORTY_EIGHT,
    }
}
pub struct KernelMemoryMap([(&'static str, usize, usize, EntryFlags); 16]);

impl Debug for KernelMemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        print_title!(f, "Kernel Space Identity Map");

        for (msg, start, end, flags) in self.0 {
            if msg.len() > 0 {
                println!(f, "{}: 0x{:x}-0x{:x} {:?}", msg, start, end, flags);
            }
        }
        Ok(())
    }
}

pub unsafe fn init_global_pages(
    layout: &LinkerLayout,
    page_allocator: &mut PageAllocator<PAGE_SIZE>,
    trap_stack: usize,
    kernel_heap_info: HeapInfo,
) -> KernelMemoryMap {
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
    let kernel_page_table = {
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

    let kernel_page_table = KMEM_PAGE_TABLE.as_mut().unwrap();
    if !set_translation_table(TableTypes::Sv39, kernel_page_table) {
        panic!("address translation not supported on this processor.");
    }

    // set up mmu satp value; todo: do this elsewhere / via zst interface
    let root_ppn = (kernel_page_table as *const _ as usize) >> 12;
    let satp_val = 8 << 60 | root_ppn;
    let descriptor = { get_global_descriptor() };
    let mut kernel_zalloc =
        |count: usize| -> Option<*mut u8> { page_allocator.zalloc(count).map(|p| p as *mut u8) };
    {
        let global_trapframe_address = {
            let frame: &mut TrapFrame = &mut trap::GLOBAL_TRAPFRAMES[0];
            frame.satp = satp_val;
            frame.trap_stack = (trap_stack + PAGE_SIZE) as *mut _;
            let global_trapframe_address = frame as *mut TrapFrame as usize;

            // store for use in trap handler
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

    for (msg, start, end, flags) in kernel_memory_map {
        if msg.len() > 0 {
            kernel_page_table.identity_map(descriptor, start, end, flags, &mut kernel_zalloc);
        }
    }

    asm!("csrw satp, {}", in(reg) satp_val);

    KernelMemoryMap(kernel_memory_map)
}
