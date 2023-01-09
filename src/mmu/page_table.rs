pub mod descriptor;
pub mod forty_eight;
pub mod thirty_nine;
pub mod thirty_two;
pub mod untyped;

pub enum PageTableEntryKind<T> {
    Branch(T),
    Leaf(T),
}

pub trait PageTable {
    type VirtualPointer;
    type Entry;
    fn entry(&self, address: Self::VirtualPointer) -> Self::Entry;
}
