use super::descriptor::{BitGroup, PageTableDescriptor};

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
