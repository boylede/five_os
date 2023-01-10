use core::cmp::Ordering;

use crate::{
    mmu::{
        page_table::{
            forty_eight::SV_FORTY_EIGHT, thirty_nine::SV_THIRTY_NINE, thirty_two::SV_THIRTY_TWO,
        },
    },
};

use super::{
    entry::{PTEntryRead, PTEntryWrite},
    page_table::{
        descriptor::PageTableDescriptor,
        untyped::{entry::PageTableEntryUntyped, PageTableUntyped},
    },
    EntryFlags, Page, PageSize, TableTypes,
};

fn translate_address(page_table: &PageTableUntyped, virtual_address: usize) -> usize {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => virtual_address,
            Sv32 => traverse_root(page_table, virtual_address, &SV_THIRTY_TWO),
            Sv39 => traverse_root(page_table, virtual_address, &SV_THIRTY_NINE),
            Sv48 => traverse_root(page_table, virtual_address, &SV_FORTY_EIGHT),
        }
    }
}

fn traverse_root(
    table: &PageTableUntyped,
    virtual_address: usize,
    descriptor: &PageTableDescriptor,
) -> usize {
    let level = descriptor.levels - 1;
    traverse(table, virtual_address, level, descriptor)
}

/// Convert a virtual address to a physical address per the algorithm
/// documented in 4.3.2 of the riscv priviliged isa spec.
fn traverse(
    table: &PageTableUntyped,
    virtual_address: usize,
    level: usize,
    descriptor: &PageTableDescriptor,
) -> usize {
    // decompose page table descriptor
    let (table_size, pte_size, vpn_segments, ppn_segments, pa_segments, levels) = {
        let d = descriptor;
        (
            d.size,
            d.entry_size,
            d.virtual_segments,
            d.page_segments,
            d.page_segments,
            d.levels,
        )
    };

    // 1) let a be satp.ppn * PAGESIZE. we are disregarding this and using the provided address as the table to search.
    let a = table as *const _ as usize;
    // and let i be LEVELS - 1, again we are disregarding this subtraction and taking it as input
    let i = level;

    // 2) let pte be the value of the page table entry at address a + va.vpn[i]*PTESIZE
    let va_vpni = extract_bits(virtual_address, &vpn_segments[level]);
    let pte: (&PageTableEntryUntyped, &PageTableDescriptor) = unsafe {
        // SAFETY: we are converting an arbitrary memory address to a usize reference,
        // so we need to be sure that the memory address is a) initialized,
        // b) contents valid for usize, c) aligned for usize, and d) no concurrent
        // access to this address can modify it.
        // a: this page table was allocated with zalloc, so the memory is known zero, or was written since then by
        // b: all initialized memory is valid for integral types
        // c: the address is aligned for usize because a is aligned for usize,
        // and the offset (va_vpni) is scaled by pte_size which represents the required alignment
        // d: we cannot prove this yet, but we are single-threaded at the moment so
        // when we switch to multi-threaded we will need to protect page tables with a mutex, semaphore or simular structure

        let entry_offset = va_vpni * pte_size;
        // check that va_vpni does not push us past the end of the table
        // this check *should* be redundant because the page table descriptor "vpn_segments"
        // passed to extract_bits should ensure only values of a limited magnitude can be
        // returned from that function, but we will check here to be sure
        assert!(entry_offset <= table_size - pte_size);
        // ((a + entry_offset) as *const usize).as_ref().unwrap()
        // GenericPageTableEntry::at_address(a + entry_offset)
        let pteu = unsafe {
            ((a + entry_offset) as *const PageTableEntryUntyped)
                .as_ref()
                .unwrap()
        };
        (pteu, &descriptor)
    };
    // 3) if page table valid bit not set, or if read/write bits set inconsistently, stop
    if !pte.extract_flags().is_valid() || pte.extract_flags().is_invalid() {
        panic!("invalid page table entry");
    }
    // 4) now we know the entry is valid, check if it is readable or executable. if not, it is a branch
    // if it is a leaf, proceed to step 5, otherwise decrement i, checking that i wasn't 0 first,
    // and continue from step 2 after setting a to the next page table based on this pte
    if pte.extract_flags().is_readable() || pte.extract_flags().is_executable() {
        // 5) pte is a leaf.
        // spec describes checking if the memory access is allowed, but that is for the hardware implementation
        // we will just return the address
        // 6) if i > 0, and the appropriate low bits in the pte are not zeroed, this is misaligned
        if pte.extract_segment(level) != 0 {
            // if extract_bits(pte.raw(), &ppn_segments[level]) != 0 {}
            panic!("invalid page table entry");
        }

        // 7) this step manages the access and dirty bits in the pte, which is again only relevent to the hardware implementation
        // 8) ready to form physical address (pa)
        // pa.pgoff = va.pgoff
        let mut pa = virtual_address & ((1 << 12) - 1);
        // if i > 0, this is a super page and the low bits of pa.ppn come from the vpn (e.g. the bits in sections i-1 thru 0)
        if i > 0 {
            for j in 0..i {
                put_bits(virtual_address, &mut pa, &vpn_segments[j], &pa_segments[j]);
            }
        }
        // the highest bits of pa.ppn come from the pte (e.g. the bits in sections LEVELS-1 thru i)
        for k in i..levels - 1 {
            pa |= pte.extract_segment(k) as usize;
            // put_bits(pte.raw(), &mut pa, &ppn_segments[k], &pa_segments[k]);
        }
        pa
    } else {
        // pte is a branch, descend to next level
        if level == 0 {
            panic!("invalid page table entry");
        }
        // combine all ppn segments from the page table entry descriptor
        // let ppn_descriptor: BitGroup = collapse_descriptor(ppn_segments);

        // let next_table = extract_bits(pte.raw(), &ppn_descriptor) << 12;
        let next_table = unsafe {
            // SAFETY: we are converting an arbitrary usize to a PageTable reference, so we need
            // to be sure that the memory address is a) initialized, b) contents valid
            // for PageTable, c) aligned for PageTable, and d) no concurrent access to this
            // address can modify it.
            // a: page tables are created with zalloc, so are always initialized
            // b: PageTable is an array of integral types, with total size equal to a memory
            // page, since the page was zero'd and since initialized memory is valid for all integral types, we are valid for PageTable
            // c: we are shifting the output of extract_bits by 12, which ensures that the low 12 bits are zero, as required
            // d: again, this will need to be protected by a mutex or semaphore, once we add support for multiple cores
            (pte.address() as *const PageTableUntyped).as_ref().unwrap()
            // pte.child_table(descriptor)
        };
        traverse(next_table, virtual_address, level - 1, descriptor)
    }
}

/// takes bits described in from_segment from "from" and writes them to "to" according to descriptor "to_segment"
fn put_bits(
    from: usize,
    to: &mut usize,
    from_segment: &(usize, usize),
    to_segment: &(usize, usize),
) {
    let mut bits = extract_bits(from, from_segment);
    let (bit_width, offset) = to_segment;
    let mask = ((1 << bit_width) - 1) << offset;
    bits = (bits << offset) & mask;
    *to &= !mask;
    *to |= bits;
}

// snipped from untyped.rs page table
fn extract_bits(from: usize, from_segment: &(usize, usize)) -> usize {
    todo!()
}

fn unmap_subtables(table: &mut PageTableUntyped, dealloc: fn(*mut u8)) {
    unsafe {
        use TableTypes::*;
        match PAGE_TABLE_TYPE {
            None => (),
            Sv32 => unmap_root(table, &SV_THIRTY_TWO, dealloc),
            Sv39 => unmap_root(table, &SV_THIRTY_NINE, dealloc),
            Sv48 => unmap_root(table, &SV_FORTY_EIGHT, dealloc),
        }
    }
}

fn unmap_root(table: &mut PageTableUntyped, descriptor: &PageTableDescriptor, dealloc: fn(*mut u8)) {
    unmap(table, descriptor, descriptor.levels - 1, dealloc)
}

fn unmap(table: &mut PageTableUntyped, descriptor: &PageTableDescriptor, level: usize, dealloc: fn(*mut u8)) {
    for index in 0..descriptor.size {
        let mut entry = (table.entry_mut(index, descriptor.entry_size), descriptor);
        // let entry = unsafe {
        //     (((&mut (table.0).0 as *mut [u8; 4096]) as *mut usize)
        //         .add(index * descriptor.entry_size))
        //     .as_mut()
        //     .unwrap()
        // };
        if entry.extract_flags().is_branch() {
            if level != 0 {
                let page = entry.address() as usize;
                // let page = extract_bits(entry.raw(), &descriptor.page_segments[level]) << 12;
                let next_table = unsafe { (page as *mut PageTableUntyped).as_mut().unwrap() };
                unmap(next_table, descriptor, level - 1, dealloc);
            } else {
                panic!("invalid page entry encountered");
            }
        }
        entry.invalidate();
    }
    if level != descriptor.levels - 1 {
        dealloc((table as *mut PageTableUntyped) as *mut u8);
    }
}

// fn map_address(
//     root: &mut PageTableUntyped,
//     virtual_address: usize,
//     physical_address: usize,
//     flags: EntryFlags,
//     page_size: PageSize,
// ) {
//     unsafe {
//         use TableTypes::*;
//         match PAGE_TABLE_TYPE {
//             None => [0; 4], //todo: remove
//             Sv32 => map_root(
//                 root,
//                 virtual_address,
//                 physical_address,
//                 flags,
//                 page_size,
//                 &SV_THIRTY_TWO,
//             ),
//             Sv39 => map_root(
//                 root,
//                 virtual_address,
//                 physical_address,
//                 flags,
//                 page_size,
//                 &SV_THIRTY_NINE,
//             ),
//             Sv48 => map_root(
//                 root,
//                 virtual_address,
//                 physical_address,
//                 flags,
//                 page_size,
//                 &SV_FORTY_EIGHT,
//             ),
//         };
//     }
// }
