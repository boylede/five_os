pub trait VirtualAddress {
    fn extract_page_index(&self, level: usize) -> u64;
    fn as_u64(&self) -> u64;
}
