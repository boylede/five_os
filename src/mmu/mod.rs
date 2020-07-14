use crate::layout::StaticLayout;
use crate::page::{PAGE_SIZE, align_address};
use crate::{print, println};

mod thirty_two;
mod thirty_nine;

pub use thirty_two::Sv32Table;
pub use thirty_nine::Sv39Table;

/// Abstraction over any MMU-backed page table type
pub struct PageTable(Page);



trait Table {
    type Entry;
    type Address;
    fn len() -> usize;
    fn initialize(base_address: usize) -> Self;
    fn get_physical_address(&self, address: usize) -> usize;
}

trait Address {
    fn to_physical(&self) -> usize;
}

pub fn setup() -> *mut Sv39Table {
    let layout = StaticLayout::get();
    println!("heap_start is {:x}", layout.heap_start);
    let mut satp = crate::cpu_status::Satp::from_address(layout.heap_start);
    println!(
        "resulting address is {:x}",
        align_address(layout.heap_start)
    );
    satp.set_mode(8);
    crate::cpu_status::set_satp(&satp);
    let table = Sv39Table::at_address(satp.address());
    table
}

pub fn test() {
    assert!(core::mem::size_of::<thirty_two::Sv32Entry>() == 4);
    assert!(core::mem::size_of::<thirty_nine::Sv39Entry>() == 8);
}
