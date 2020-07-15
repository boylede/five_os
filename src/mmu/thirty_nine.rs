use crate::mmu::{BitGroup, PageTableDescriptor};

pub(super) const SV_THIRTY_NINE: PageTableDescriptor = PageTableDescriptor {
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
