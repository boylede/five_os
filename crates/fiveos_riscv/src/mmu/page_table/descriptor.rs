use super::PageTableKind;

pub struct PageTableDescriptor {
    /// the size of the page table, in bytes (always 4096)
    pub size: usize,
    /// the number of levels of page tables
    pub levels: usize,
    /// the size of an entry, in bytes
    pub entry_size: usize,
    /// description of the "virtual page number" field of virtual addresses
    pub virtual_segments: &'static [BitGroup],
    /// description of the "physical page number" field of page table entries
    pub page_segments: &'static [BitGroup],
    /// description of the "physical page number" field of physical addresses
    pub physical_segments: &'static [BitGroup],
}

impl PageTableDescriptor {
    pub fn virtual_address_size(&self) -> usize {
        collapse(self.virtual_segments).0 + 12
    }
    pub fn physical_address_size(&self) -> usize {
        collapse(self.physical_segments).0 + 12
    }
}

// #[derive(Debug)]
// pub struct DescribedEntry(u8);

// impl PageTableKind for PageTableDescriptor {
//     type Entry = DescribedEntry;
//     fn size(&self) -> usize {
//         self.size
//     }
//     fn depth(&self) -> usize {
//         self.levels
//     }
//     fn entry_size(&self) -> usize {
//         self.entry_size
//     }
//     fn entry_segments(&self) -> &[BitGroup] {
//         self.page_segments
//     }
//     fn physical_segments(&self) -> &[BitGroup] {
//         self.physical_segments
//     }
//     fn virtual_segments(&self) -> &[BitGroup] {
//         self.virtual_segments
//     }
// }

/// A (size, offset) where size is # of bits and offset is
/// the bit address of the lowest bit in the group.
pub type BitGroup = (usize, usize);

pub fn collapse(segments: &[BitGroup]) -> BitGroup {
    let size = segments.iter().map(|(bits, _)| *bits).sum();
    (size, segments[0].1)
}
