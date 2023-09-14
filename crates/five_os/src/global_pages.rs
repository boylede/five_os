use core::{arch::asm, fmt::Debug};
use fiveos_allocator::page::PageAllocator;

use crate::{kernel_heap::HeapInfo, layout::LinkerLayout};
use five_os::{trap::TrapFrame, *};
use fiveos_peripherals::{print, print_title, println};
use fiveos_riscv::mmu::{
    page_table::{thirty_nine::Sv39, AnyPageTable, PageTable, PAGE_SIZE},
    set_translation_table, EntryFlags, TableTypes,
};
use fiveos_virtio::{
    clint::{CLINT_BASE_ADDRESS, CLINT_END_ADDRESS},
    plic::{PLIC_BASE_ADDRESS, PLIC_END_ADDRESS},
    uart::{UART_BASE_ADDRESS, UART_END_ADDRESS},
};

/// MMU page table for kernel
static mut KMEM_PAGE_TABLE: AnyPageTable = AnyPageTable::Off;

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

    /////////////////////////////////////////////////////////////////////////////////////////
    // initialize kernel root page table
    /////////////////////////////////////////////////////////////////////////////////////////
    let kpt = page_allocator.zalloc(1).unwrap() as *const u8;
    let kpta = kpt as *const usize as usize;
    KMEM_PAGE_TABLE = AnyPageTable::Sv39(PageTable::in_place(kpt, Sv39));
    let kpt = &KMEM_PAGE_TABLE;

    /////////////////////////////////////////////////////////////////////////////////////////
    // setup SATP
    /////////////////////////////////////////////////////////////////////////////////////////
    let satp_val = 8 << 60 | (kpta >> 12);
    if !set_translation_table(TableTypes::Sv39, kpta) {
        panic!("address translation not supported on this processor.");
    }

    /////////////////////////////////////////////////////////////////////////////////////////
    // add kernel dynamic memory info to memory map
    /////////////////////////////////////////////////////////////////////////////////////////
    {
        kernel_memory_map[0] = ("Kernel Root Page Table", kpta, kpta, EntryFlags::READ_WRITE);
        let HeapInfo { start, end, .. } = kernel_heap_info;
        kernel_memory_map[1] = ("Kernel Dynamic Memory", start, end, EntryFlags::READ_WRITE);
    }
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

        match kpt {
            AnyPageTable::Sv39(kpt) => {
                kpt.identity_map(
                    global_trapframe_address,
                    global_trapframe_address + PAGE_SIZE,
                    EntryFlags::READ_WRITE,
                    &mut kernel_zalloc,
                );
                for (msg, start, end, flags) in kernel_memory_map {
                    if msg.len() > 0 {
                        kpt.identity_map(start, end, flags, &mut kernel_zalloc);
                    }
                }
            }
            _ => todo!(),
        }
    }
    KernelMemoryMap(kernel_memory_map)
}
