use crate::mmu::{BitGroup, PageTableDescriptor};

pub(super) const SV_FORTY_EIGHT: PageTableDescriptor = PageTableDescriptor {
    size: PAGESIZE,
    levels: LEVELS,
    entry_size: PTESIZE,
    virtual_segments: &VPN_SEGMENTS as &[(_, _)],
    page_segments: &PPN_SEGMENTS as &[(_, _)],
    physical_segments: &PA_SEGMENTS as &[(_, _)],
};

/// sv48 constant for page table traversal
const LEVELS: usize = 4;
/// sv48 constant for page table traversal
const PTESIZE: usize = 8;
/// sv48 constant for page table traversal
const PAGESIZE: usize = 1 << 12;
/// description of the "virtual page number" field of sv48 virtual addresses
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const VPN_SEGMENTS: [BitGroup; LEVELS] = [(9, 12), (9, 21), (9, 30), (9, 39)];
/// description of the "physical page number" field of sv48 page table entries
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const PPN_SEGMENTS: [BitGroup; LEVELS] = [(9, 10), (9, 19), (9, 28), (17, 37)];
/// description of the "physical page number" field of sv48 physical addresses
/// used in page table traversal function, each tuple describes one group of
/// bits as (size, offset) where size is # of bits and offset is the bit address
/// of the lowest bit in the group.
const PA_SEGMENTS: [BitGroup; LEVELS] = [(9, 12), (9, 21), (9, 30), (17, 39)];
