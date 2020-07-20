use super::PageTableDescriptor;
use super::PageTable;
use super::collapse_descriptor;

pub struct Entry(usize);

impl Entry {
    /// check lowest bit is set
    pub fn is_valid(&self) -> bool {
        self.0 & 0b1 == 1
    }

    /// checks read & write bits not inconsistant
    pub fn is_invalid(&self) -> bool {
        self.is_readable() && !self.is_writable()
    }

    pub fn invalidate(&mut self) {
        self.0 = 0;
    }

    /// checks bit 1 is set
    pub fn is_readable(&self) -> bool {
        self.0 & 0b10 == 0b10
    }

    /// checks bit x is set
    pub fn is_writable(&self) -> bool {
        self.0 & 0b100 == 0b100
    }

    /// checks bit 3 is set
    pub fn is_executable(&self) -> bool {
        self.0 & 0b1000 == 0b1000
    }

    pub fn is_branch(&self) -> bool {
        self.is_valid() && !self.is_readable() && !self.is_executable() && !self.is_writable()
    }

    /// produce a page table entry based on the provided descriptor,
    /// permissions bits, and software bits, and sets valid bit
    pub(in self) fn new(address: usize, flags: EntryFlags, descriptor: &PageTableDescriptor) -> Self {
        let mut bits = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            bits = (address << offset) & mask;
        }
        bits |= flags.to_entry();
        Entry(bits)
    }
    pub fn raw(&self) -> usize {
        self.0
    }
    pub fn set(&mut self, new: usize) {
        self.0 = new;
    }
    pub(in super) fn get_address(&self, descriptor: &PageTableDescriptor) -> usize {
        let mut address = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (self.0 & mask) >> offset;
        }
        address << 12
    }
    pub(in super) fn set_with(&mut self, address: usize, flags: EntryFlags, descriptor: &PageTableDescriptor) {
        let address = address >> 12;
        let mut bits = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            bits = (address << offset) & mask;
        }
        bits |= flags.to_entry();
        self.0 = bits;
    }
    pub fn from_raw(entry: usize) -> Self {
        Entry(entry)
    }
    pub(super) unsafe fn at_address<'a>(address: usize) -> &'a Self {
        (address as *const Entry).as_ref().unwrap()
    }
    pub(super) unsafe fn at_address_mut<'a>(address: usize) -> &'a mut Self {
        (address as *mut Entry).as_mut().unwrap()
    }
    pub(super) unsafe fn child_table(&self, descriptor: &PageTableDescriptor) -> &PageTable {
        let mut address = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (self.0 << offset) & mask;
        }
        address <<= 12;
        (address as *const PageTable).as_ref().unwrap()
    }
    pub(super) unsafe fn child_table_mut(&mut self, descriptor: &PageTableDescriptor) -> &mut PageTable {
        let mut address = 0;
        for level in 0..descriptor.levels {
            let (bit_width, offset) = descriptor.page_segments[level];
            let mask = ((1 << bit_width) - 1) << offset;
            address = (self.0 << offset) & mask;
        }
        address <<= 12;
        (address as *mut PageTable).as_mut().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct EntryFlags {
    permissions: PermFlags,
    software: SoftFlags,
    user: bool,
    global: bool,
}

impl EntryFlags {
    /// Puts each bitflag into the lower 9 bits of a usize,
    /// ready for insertion into any type of page table entry
    /// also sets valid flag
    pub fn to_entry(&self) -> usize {
        0b1 | // set valid bit
        (self.permissions as usize) << 1 |
        (self.user as usize) << 4 |
        (self.global as usize) << 5
    }
    pub fn software(&mut self) -> &mut SoftFlags {
        &mut self.software
    }
}

/// Permissions flags in a page table entry.
#[derive(Clone, Copy)]
enum PermFlags {
    Leaf = 0b000,
    ReadOnly = 0b001,
    ReadWrite = 0b011,
    ReadWriteExecute = 0b111,
    ReadExecute = 0b101,
}

/// Placeholder for the 2 software-defined bits allowed in the mmu's page table entries
#[derive(Clone, Copy, Default)]
pub struct SoftFlags(u8);

impl SoftFlags {
    fn set(&mut self, value: u8) {
        self.0 = value & 0b11;
    }
    fn get(&self) -> u8 {
        self.0
    }
    fn clear(&mut self) {
        self.0 = 0
    }
    fn set_a(&mut self) {
        self.0 |= 0b01
    }
    fn set_b(&mut self) {
        self.0 |= 0b10
    }
    fn get_a(&mut self) -> bool {
        self.0 & 0b01 == 0b01
    }
    fn get_b(&mut self) -> bool {
        self.0 & 0b10 == 0b10
    }
}
