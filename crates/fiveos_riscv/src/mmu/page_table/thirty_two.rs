use core::sync::atomic::AtomicU32;

use crate::mmu::entry::PTEntry;

use super::{
    descriptor::{BitGroup, PageTableDescriptor},
    PageTableKind,
};

pub const SV_THIRTY_TWO: PageTableDescriptor = PageTableDescriptor {
    size: PAGESIZE,
    levels: LEVELS,
    entry_size: PTESIZE,
    virtual_segments: &VPN_SEGMENTS as &[(_, _)],
    page_segments: &PPN_SEGMENTS as &[(_, _)],
    physical_segments: &PA_SEGMENTS as &[(_, _)],
};

/// sv39 constant for page table traversal
const LEVELS: usize = 2;
/// sv39 constant for page table traversal
const PTESIZE: usize = 4;
/// sv39 constant for page table traversal
const PAGESIZE: usize = 1 << 12;
/// description of the "virtual page number" field of sv32 virtual addresses
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const VPN_SEGMENTS: [BitGroup; LEVELS] = [(10, 12), (10, 22)];
/// description of the "physical page number" field of sv32 page table entries
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const PPN_SEGMENTS: [BitGroup; LEVELS] = [(10, 10), (12, 20)];
/// description of the "physical page number" field of sv32 physical addresses
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const PA_SEGMENTS: [BitGroup; LEVELS] = [(10, 12), (12, 22)];

/// ZST to tag Sv32-type page tables
#[derive(Debug, Clone, Copy)]
pub struct Sv32;

impl PageTableKind for Sv32 {
    type Entry = Entry;
    fn size(&self) -> usize {
        PAGESIZE
    }

    fn depth(&self) -> usize {
        LEVELS
    }

    fn entry_size(&self) -> usize {
        PTESIZE
    }

    fn entry_segments(&self) -> &[BitGroup] {
        &PPN_SEGMENTS
    }

    fn physical_segments(&self) -> &[BitGroup] {
        &PA_SEGMENTS
    }

    fn virtual_segments(&self) -> &[BitGroup] {
        &VPN_SEGMENTS
    }
}

/// Sv32 Page Table Entry
#[derive(Debug)]
#[repr(transparent)]
pub struct Entry(AtomicU32);

impl Entry {}

impl PTEntry for Entry {
    fn read_flags(&self) -> crate::mmu::EntryFlags {
        todo!()
    }

    fn read_address(&self) -> u64 {
        todo!()
    }

    fn read_extended_flags(&self) -> crate::mmu::entry::ExtendedFlags {
        todo!()
    }

    fn extract_segment(&self, level: usize) -> u64 {
        todo!()
    }

    fn load(&self) -> u64 {
        todo!()
    }

    fn write(&self, old_value: u64, address: u64, flags: crate::mmu::EntryFlags) -> bool {
        todo!()
    }

    fn invalidate(&self, old_value: u64) -> bool {
        todo!()
    }
}
