use core::sync::atomic::{AtomicU64, Ordering};

use crate::mmu::{entry::PTEntry, EntryFlags};

use super::{
    descriptor::{BitGroup, PageTableDescriptor},
    PageTableKind,
};

pub const SV_THIRTY_NINE: PageTableDescriptor = PageTableDescriptor {
    size: PAGESIZE,
    levels: LEVELS,
    entry_size: PTESIZE,
    virtual_segments: &VPN_SEGMENTS as &[(_, _)],
    page_segments: &PPN_SEGMENTS as &[(_, _)],
    physical_segments: &PA_SEGMENTS as &[(_, _)],
};

/// sv39 constant for page table traversal
const LEVELS: usize = 3;
/// sv39 constant for page table traversal
const PTESIZE: usize = 8;
/// sv39 constant for page table traversal
const PAGESIZE: usize = 1 << 12;
/// description of the "virtual page number" field of sv39 virtual addresses
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const VPN_SEGMENTS: [BitGroup; LEVELS] = [(9, 12), (9, 21), (9, 30)];
/// description of the "physical page number" field of sv39 page table entries
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const PPN_SEGMENTS: [BitGroup; LEVELS] = [(9, 10), (9, 19), (26, 28)];
/// description of the "physical page number" field of sv39 physical addresses
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const PA_SEGMENTS: [BitGroup; LEVELS] = [(9, 12), (9, 21), (26, 30)];
// const FULL_PPN: BitGroup = (44, 10);

/// ZST to tag Sv39-type page tables
#[derive(Debug, Clone, Copy)]
pub struct Sv39;

impl PageTableKind for Sv39 {
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

/// Sv39 Page Table Entry
#[derive(Debug)]
#[repr(transparent)]
pub struct Entry(AtomicU64);

impl Entry {}
impl PTEntry for Entry {
    fn read_flags(&self) -> EntryFlags {
        let value = self.0.load(Ordering::Relaxed);
        EntryFlags::from_u16((value & (1 << 10) - 1) as u16)
    }

    fn read_address(&self) -> u64 {
        let entry = self.load();
        let mut address = 0;
        for level in 0..LEVELS {
            let (bit_width, offset) = PPN_SEGMENTS[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (entry & mask) >> offset;
        }
        let address = address << 12;
        // let mut uart = unsafe {Uart0::new()};
        // println!(uart, "extracted address {:x}", address);
        address
    }

    fn read_extended_flags(&self) -> crate::mmu::entry::ExtendedFlags {
        todo!()
    }

    fn extract_segment(&self, level: usize) -> u64 {
        todo!()
    }

    fn load(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    fn write(&self, old_value: u64, address: u64, flags: EntryFlags) -> bool {
        let address = address >> 12;
        let mut bits = 0;
        for level in 0..LEVELS {
            let (bit_width, offset) = PPN_SEGMENTS[level];
            let mask = ((1 << bit_width) - 1) << offset;
            bits = (address << offset) & mask;
        }
        let new_value = flags.as_u16() as u64 | bits as u64;

        self.0
            .compare_exchange(old_value, new_value, Ordering::Release, Ordering::Relaxed)
            .is_ok()
    }

    fn invalidate(&self, old_value: u64) -> bool {
        todo!()
    }
}
