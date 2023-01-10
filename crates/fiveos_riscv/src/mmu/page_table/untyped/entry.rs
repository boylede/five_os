use crate::mmu::{
    entry::{ExtendedFlags, PTEntryRead, PTEntryWrite},
    page_table::descriptor::PageTableDescriptor,
    EntryFlags,
};

#[repr(transparent)]
pub struct PageTableEntryUntyped(usize);

impl PTEntryRead for (&PageTableEntryUntyped, &PageTableDescriptor) {
    fn extract_flags(&self) -> EntryFlags {
        EntryFlags::from_u16((self.0 .0 & (1 << 10) - 1) as u16)
    }

    fn address_limited(&self) -> Option<u32> {
        todo!()
    }

    fn address(&self) -> u64 {
        let mut address = 0;
        for level in 0..self.1.levels {
            let (bit_width, offset) = self.1.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (self.0 .0 as u64 & mask) >> offset;
        }
        address << 12
    }

    fn extract_extended_flags(&self) -> ExtendedFlags {
        todo!()
    }

    fn extract_segment_limited(&self, level: usize) -> u32 {
        todo!()
    }
    /// given the page table level (depth), extract the bits corresponding to this level in this entry,
    /// and return them according to the bit positions of the virtual address
    fn extract_segment(&self, level: usize) -> u64 {
        todo!()
    }
}

impl PTEntryRead for (&mut PageTableEntryUntyped, &PageTableDescriptor) {
    fn extract_flags(&self) -> EntryFlags {
        EntryFlags::from_u16((self.0 .0 & (1 << 10) - 1) as u16)
    }

    fn address_limited(&self) -> Option<u32> {
        todo!()
    }

    fn address(&self) -> u64 {
        let mut address = 0;
        for level in 0..self.1.levels {
            let (bit_width, offset) = self.1.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (self.0 .0 as u64 & mask) >> offset;
        }
        address << 12
    }

    fn extract_extended_flags(&self) -> ExtendedFlags {
        todo!()
    }

    fn extract_segment_limited(&self, level: usize) -> u32 {
        todo!()
    }
    /// given the page table level (depth), extract the bits corresponding to this level in this entry,
    /// and return them according to the bit positions of the virtual address
    fn extract_segment(&self, level: usize) -> u64 {
        todo!()
    }
}

impl PTEntryWrite for (&mut PageTableEntryUntyped, &PageTableDescriptor) {
    fn write_flags(&mut self, flags: EntryFlags) {
        self.0 .0 |= flags.as_u16() as usize;
    }
    fn invalidate(&mut self) {
        todo!()
    }
    fn write_address(&mut self, address: u64) {
        let address = address >> 12;
        let mut bits = 0;
        for level in 0..(self.1.levels) {
            let (bit_width, offset) = self.1.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            bits = (address << offset) & mask;
        }
        self.0 .0 |= bits as usize;
    }

    fn write_extended_flags(&mut self) -> ExtendedFlags {
        todo!()
    }
}

impl PageTableEntryUntyped {
    #[inline]
    pub const fn copy_flags(&self) -> EntryFlags {
        // write lower 10 bits of entry into EntryFlags
        EntryFlags::from_u16((self.0 & (1 << 10) - 1) as u16)
    }
    /// clear lowest bit
    pub const fn invalidate(&mut self) {
        self.0 &= !1;
    }

    // /// produce a page table entry based on the provided descriptor,
    // /// permissions bits, and software bits, and sets valid bit
    // pub(self) fn new(address: usize, flags: EntryFlags, descriptor: &PageTableDescriptor) -> Self {
    //     let mut bits = 0;
    //     for level in 0..descriptor.levels {
    //         let (bit_width, offset) = descriptor.page_segments[level];
    //         let mask = ((1 << bit_width) - 1) << offset;
    //         bits = (address << offset) & mask;
    //     }
    //     bits |= flags.as_u16() as usize;
    //     GenericPageTableEntry(bits)
    // }
    // pub fn raw(&self) -> usize {
    //     self.0
    // }
    // pub fn set(&mut self, new: usize) {
    //     self.0 = new;
    // }
    pub(super) fn get_address(&self, descriptor: &PageTableDescriptor) -> usize {
        let mut address = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (self.0 & mask) >> offset;
        }
        address << 12
    }
    pub(super) fn set_with(
        &mut self,
        address: usize,
        flags: EntryFlags,
        descriptor: &PageTableDescriptor,
    ) {
        // print!("setting entry with address {:x} ->", address);
        let address = address >> 12;
        let mut bits = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            bits = (address << offset) & mask;
        }
        bits |= flags.as_u16() as usize;
        // println!("{:x}", bits);
        self.0 = bits;
    }
    // pub fn from_raw(entry: usize) -> Self {
    //     GenericPageTableEntry(entry)
    // }
    // pub(super) unsafe fn at_address<'a>(address: usize) -> &'a Self {
    //     (address as *const GenericPageTableEntry).as_ref().unwrap()
    // }
    // pub(super) unsafe fn at_address_mut<'a>(address: usize) -> &'a mut Self {
    //     (address as *mut GenericPageTableEntry).as_mut().unwrap()
    // }
    // pub(super) unsafe fn child_table(&self, descriptor: &PageTableDescriptor) -> &GenericPageTable {
    //     let mut address = 0;
    //     for level in 0..descriptor.levels {
    //         let (bit_width, offset) = descriptor.page_segments[level];
    //         let mask = ((1 << bit_width) - 1) << offset;
    //         address = (self.0 << offset) & mask;
    //     }
    //     address <<= 12;
    //     (address as *const GenericPageTable).as_ref().unwrap()
    // }
    // pub(super) unsafe fn child_table_mut(
    //     &mut self,
    //     descriptor: &PageTableDescriptor,
    // ) -> &mut GenericPageTable {
    //     let mut address = 0;
    //     for level in 0..descriptor.levels {
    //         let (bit_width, offset) = descriptor.page_segments[level];
    //         let mask = ((1 << bit_width) - 1) << offset;
    //         address = (self.0 << offset) & mask;
    //     }
    //     address <<= 12;
    //     (address as *mut GenericPageTable).as_mut().unwrap()
    // }
}

fn write_char(b: bool, c: char, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    if b {
        write!(f, "{}", c)
    } else {
        write!(f, " ")
    }
}

impl core::fmt::Debug for PageTableEntryUntyped {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.copy_flags().is_valid() {
            let user = self.copy_flags().is_user();
            let global = self.copy_flags().is_global();
            let (a, b) = self.copy_flags().read_softflags();
            let accessed = self.copy_flags().is_accessed();
            let dirty = self.copy_flags().is_dirty();
            if self.copy_flags().is_branch() {
                write!(f, "branch")
            } else {
                write_char(self.copy_flags().is_readable(), 'r', f)?;
                write_char(self.copy_flags().is_writable(), 'w', f)?;
                write_char(self.copy_flags().is_executable(), 'e', f)?;
                write!(f, "-")?;
                write_char(user, 'U', f)?;
                write_char(global, 'G', f)?;
                write_char(accessed, 'A', f)?;
                write_char(dirty, 'D', f)?;
                write!(f, "-")?;
                write_char(a, 'a', f)?;
                write_char(b, 'b', f)?;
                write!(f, "-")
            }
        } else {
            write!(f, "not mapped.")
        }
    }
}
