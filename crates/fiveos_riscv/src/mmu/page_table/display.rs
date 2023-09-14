use core::fmt::Display;

use crate::mmu::entry::PTEntry;

use super::{PageTable, PageTableKind};

impl<K> Display for PageTable<K>
where
    K: PageTableKind + core::fmt::Debug + Copy,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        inner(self, 0, 0, f);
        Ok(())
    }
}

fn inner<K>(
    this: &PageTable<K>,
    base_address: usize,
    descent: usize,
    f: &mut core::fmt::Formatter<'_>,
) where
    K: PageTableKind + core::fmt::Debug + Copy,
{
    let max_bits = this.kind.virtual_address_size();
    let bits_known: usize = this
        .kind
        .virtual_segments()
        .iter()
        .take(descent + 1)
        .map(|(bits, _)| *bits)
        .sum();
    let bits_unknown = max_bits - bits_known;
    let page_size = 1 << bits_unknown;
    write!(
        f,
        "Reading pagetable located at 0x{:x}:",
        this as *const _ as usize
    )
    .unwrap();

    for index in 0..this.kind.size() / this.kind.entry_size() {
        let resulting_address = base_address + (index * page_size);
        let entry = this.entry(index);
        let flags = entry.read_flags();
        if flags.is_valid() {
            write!(
                f,
                "{}-{}: 0x{:x}-0x{:x}: {:?}",
                descent,
                index,
                resulting_address,
                resulting_address + page_size - 1,
                entry
            )
            .unwrap();

            if flags.is_branch() {
                let next = entry.read_address();
                let next_table = unsafe { (next as *const PageTable<K>).as_ref().unwrap() };

                inner(next_table, resulting_address, descent + 1, f);
            }
        }
    }
}
