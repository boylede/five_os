pub mod descriptor;
pub mod display;
pub mod forty_eight;
pub mod thirty_nine;
pub mod thirty_two;
pub mod untyped;

/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
pub const PAGE_ADDR_MAGNITIDE: usize = 12;
/// size of the smallest page allocation
pub const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with low 12 bits set
pub const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

pub enum PageTableEntryKind<T> {
    Branch(T),
    Leaf(T),
}

pub trait PageTable {
    type VirtualPointer;
    type Entry;
    fn entry(&self, address: Self::VirtualPointer) -> Self::Entry;
}
