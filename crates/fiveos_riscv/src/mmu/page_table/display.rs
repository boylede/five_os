use crate::mmu::entry::PTEntryRead;

use super::{
    descriptor::PageTableDescriptor,
    untyped::{PageTableUntyped, PageTableDynamicTyped},
};

impl<'a, 'b> core::fmt::Display for PageTableDynamicTyped<'a, 'b> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let PageTableDynamicTyped(table, descriptor) = *self;
        let base_address = todo!();
        let descent = todo!();
        inner_print_map(f, table, descriptor, base_address, descent);
        Ok(())
    }
}

fn inner_print_map(
    f: &mut core::fmt::Formatter<'_>,
    table: &PageTableUntyped,
    descriptor: &PageTableDescriptor,
    base_address: usize,
    descent: usize,
) {
    let max_bits = descriptor.virtual_address_size();
    let bits_known: usize = descriptor
        .virtual_segments
        .iter()
        .take(descent + 1)
        .map(|(bits, _)| *bits)
        .sum();
    let bits_unknown = max_bits - bits_known;
    let page_size = 1 << bits_unknown;
    write!(
        f,
        "Reading pagetable located at 0x{:x}:",
        table as *const PageTableUntyped as usize
    );

    for index in 0..descriptor.size / descriptor.entry_size {
        let resulting_address = base_address + (index * page_size);
        let entry = (table.entry(index, descriptor.entry_size), descriptor);
        if entry.extract_flags().is_valid() {
            write!(
                f,
                "{}-{}: 0x{:x}-0x{:x}: {:?}",
                descent,
                index,
                resulting_address,
                resulting_address + page_size - 1,
                entry.0
            );
            
            if entry.extract_flags().is_branch() {
                let next = entry.address();
                let next_table = unsafe { (next as *const PageTableUntyped).as_ref().unwrap() };

                inner_print_map(f, next_table, descriptor, resulting_address, descent + 1);
            }
        }
    }
}
